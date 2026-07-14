//! Microphone capture and level detection.
//!
//! The original used bundled `ffmpeg` with DirectShow to record clips and
//! `volumedetect` to read `max_volume`. That side-effecting behaviour lives
//! behind the [`MicRecorder`] trait so the monitor logic can be unit-tested
//! with a fake recorder that needs no hardware.

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;

/// A recorded clip plus its measured peak level.
#[derive(Debug, Clone)]
pub struct ClipResult {
    pub max_db: f32,
    pub duration_s: u32,
}

/// Abstraction over microphone capture + level measurement.
pub trait MicRecorder {
    /// Name of the default capture device (for diagnostics).
    fn default_device(&self) -> Result<String>;
    /// Record `seconds` of mono audio to `out_file` and return its peak dB.
    fn record_clip(&self, seconds: u32, out_file: &Path) -> Result<ClipResult>;
    /// Whether the backend is usable right now (binary present, etc.).
    fn available(&self) -> bool;
}

/// ffmpeg/DirectShow-backed recorder (Windows), locating `ffmpeg.exe` under
/// `tools/` exactly like the PowerShell version, with a PATH fallback.
pub struct FfmpegMic {
    ffmpeg: Option<std::path::PathBuf>,
}

impl FfmpegMic {
    /// Discover ffmpeg under `<root>/tools/**/ffmpeg(.exe)` or on PATH.
    pub fn discover(root: &Path) -> Self {
        let bin = if cfg!(windows) {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        };
        let found = find_ffmpeg(&root.join("tools"), bin).or_else(|| which_on_path(bin));
        Self { ffmpeg: found }
    }

    fn ffmpeg_path(&self) -> Result<&Path> {
        self.ffmpeg
            .as_deref()
            .ok_or_else(|| anyhow!("ffmpeg not found under tools/ or on PATH"))
    }

    /// Parse `max_volume: -12.3 dB` out of ffmpeg's volumedetect stderr.
    fn parse_max_db(output: &str) -> f32 {
        for line in output.lines() {
            if let Some(idx) = line.find("max_volume:") {
                let rest = &line[idx + "max_volume:".len()..];
                let token: String = rest
                    .trim()
                    .chars()
                    .take_while(|c| !c.is_whitespace())
                    .collect();
                if let Ok(v) = token.parse::<f32>() {
                    return v;
                }
            }
        }
        -100.0
    }
}

impl MicRecorder for FfmpegMic {
    fn available(&self) -> bool {
        self.ffmpeg.is_some()
    }

    fn default_device(&self) -> Result<String> {
        let ffmpeg = self.ffmpeg_path()?;
        let out = Command::new(ffmpeg)
            .args([
                "-hide_banner",
                "-list_devices",
                "true",
                "-f",
                "dshow",
                "-i",
                "dummy",
            ])
            .output()
            .context("failed to run ffmpeg -list_devices")?;
        let text = String::from_utf8_lossy(&out.stderr);
        for line in text.lines() {
            // Lines look like:  "Microphone (Realtek)" (audio)
            if line.contains("(audio)") {
                if let (Some(a), Some(b)) = (line.find('"'), line.rfind('"')) {
                    if b > a + 1 {
                        return Ok(line[a + 1..b].to_string());
                    }
                }
            }
        }
        Err(anyhow!("no DirectShow audio device found"))
    }

    fn record_clip(&self, seconds: u32, out_file: &Path) -> Result<ClipResult> {
        let ffmpeg = self.ffmpeg_path()?;
        let device = self.default_device()?;
        // Record the clip.
        let status = Command::new(ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "dshow",
                "-i",
                &format!("audio={device}"),
                "-t",
                &seconds.to_string(),
                "-ac",
                "1",
                "-ar",
                "16000",
            ])
            .arg(out_file)
            .status()
            .context("failed to run ffmpeg record")?;
        if !status.success() || !out_file.exists() {
            return Err(anyhow!("recording failed: {}", out_file.display()));
        }
        // Measure peak level via volumedetect.
        let out = Command::new(ffmpeg)
            .args(["-hide_banner", "-i"])
            .arg(out_file)
            .args(["-af", "volumedetect", "-f", "null", "-"])
            .output()
            .context("failed to run ffmpeg volumedetect")?;
        let text = String::from_utf8_lossy(&out.stderr);
        Ok(ClipResult {
            max_db: Self::parse_max_db(&text),
            duration_s: seconds,
        })
    }
}

fn find_ffmpeg(dir: &Path, bin: &str) -> Option<std::path::PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let entries = std::fs::read_dir(&d).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|n| n.to_str()) == Some(bin) {
                return Some(path);
            }
        }
    }
    None
}

fn which_on_path(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(bin);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Test recorder: returns scripted dB levels and never touches hardware.
#[cfg(test)]
pub struct FakeMic {
    levels: std::cell::RefCell<std::collections::VecDeque<f32>>,
}

#[cfg(test)]
impl FakeMic {
    pub fn new(levels: Vec<f32>) -> Self {
        Self {
            levels: std::cell::RefCell::new(levels.into()),
        }
    }
}

#[cfg(test)]
impl MicRecorder for FakeMic {
    fn available(&self) -> bool {
        true
    }
    fn default_device(&self) -> Result<String> {
        Ok("FakeMic".to_string())
    }
    fn record_clip(&self, seconds: u32, _out_file: &Path) -> Result<ClipResult> {
        let db = self.levels.borrow_mut().pop_front().unwrap_or(-100.0);
        Ok(ClipResult {
            max_db: db,
            duration_s: seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_max_volume_line() {
        let sample = "[Parsed_volumedetect_0 @ 0x1] mean_volume: -30.0 dB\n\
                      [Parsed_volumedetect_0 @ 0x1] max_volume: -12.5 dB\n";
        assert!((FfmpegMic::parse_max_db(sample) - (-12.5)).abs() < f32::EPSILON);
    }

    #[test]
    fn missing_max_volume_defaults_to_silence() {
        assert!((FfmpegMic::parse_max_db("nothing here") - (-100.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn fake_mic_replays_levels() {
        let mic = FakeMic::new(vec![-5.0, -60.0]);
        assert!((mic.record_clip(2, Path::new("x")).unwrap().max_db - (-5.0)).abs() < f32::EPSILON);
        assert!(
            (mic.record_clip(2, Path::new("x")).unwrap().max_db - (-60.0)).abs() < f32::EPSILON
        );
    }
}
