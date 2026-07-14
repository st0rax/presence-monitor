//! Text-to-speech backend.
//!
//! The original used .NET `System.Speech` to synthesize a WAV and play it
//! aloud, picking a voice by culture (e.g. `fr-FR`). On Windows we drive the
//! same SAPI stack through `powershell -Command`. The behaviour sits behind
//! the [`TtsEngine`] trait so the monitor logic stays testable.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Abstraction over speaking a phrase aloud (and saving it to a WAV).
pub trait TtsEngine {
    /// Speak `text` in `lang` (culture prefix like `fr-FR`) and also write a
    /// WAV copy to `out_file` for the log.
    fn speak(&self, text: &str, lang: &str, out_file: &Path) -> Result<()>;
    /// Whether the backend can run in this environment.
    fn available(&self) -> bool;
}

/// Windows SAPI (`System.Speech`) engine driven via PowerShell.
pub struct WindowsSapiTts;

impl WindowsSapiTts {
    fn powershell_available() -> bool {
        which("powershell.exe").or_else(|| which("pwsh")).is_some()
    }

    /// Build the PowerShell script that saves a WAV and plays it aloud,
    /// mirroring the original Invoke-TTS.
    fn build_script(text: &str, lang: &str, out_file: &Path) -> String {
        let esc_text = ps_single_quote(text);
        let esc_lang = ps_single_quote(lang);
        let esc_file = ps_single_quote(&out_file.to_string_lossy());
        format!(
            "Add-Type -AssemblyName System.Speech; \
             Add-Type -AssemblyName System.Media; \
             $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
             $voice = $synth.GetInstalledVoices() | Where-Object {{ $_.VoiceInfo.Culture.Name -like ('{lang}' + '*') }} | Select-Object -First 1; \
             if ($voice) {{ try {{ $synth.SelectVoice($voice.VoiceInfo.Name) }} catch {{ }} }}; \
             $synth.SetOutputToWaveFile({file}); \
             $synth.Speak({text}); \
             $synth.Dispose(); \
             $player = New-Object System.Media.SoundPlayer({file}); \
             $player.PlaySync(); \
             $player.Dispose();",
            lang = esc_lang,
            file = esc_file,
            text = esc_text,
        )
    }
}

impl TtsEngine for WindowsSapiTts {
    fn available(&self) -> bool {
        cfg!(windows) && Self::powershell_available()
    }

    fn speak(&self, text: &str, lang: &str, out_file: &Path) -> Result<()> {
        let script = Self::build_script(text, lang, out_file);
        let shell = which("powershell.exe")
            .or_else(|| which("pwsh"))
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "powershell.exe".to_string());
        let mut cmd = Command::new(shell);
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let status = cmd
            .status()
            .context("failed to run PowerShell for TTS")?;
        if !status.success() {
            anyhow::bail!("TTS PowerShell exited with failure");
        }
        Ok(())
    }
}

/// Escape a string for embedding inside a PowerShell single-quoted literal.
fn ps_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn which(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(bin);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Test engine: records what it was asked to say, never makes noise.
#[cfg(test)]
#[derive(Default)]
pub struct RecordingTts {
    pub spoken: std::cell::RefCell<Vec<(String, String)>>,
}

#[cfg(test)]
impl TtsEngine for RecordingTts {
    fn available(&self) -> bool {
        true
    }
    fn speak(&self, text: &str, lang: &str, _out_file: &Path) -> Result<()> {
        self.spoken
            .borrow_mut()
            .push((lang.to_string(), text.to_string()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_quote_escaping_doubles_apostrophes() {
        assert_eq!(ps_single_quote("it's"), "'it''s'");
        assert_eq!(ps_single_quote("plain"), "'plain'");
    }

    #[test]
    fn build_script_embeds_escaped_values() {
        let script = WindowsSapiTts::build_script("Dors-tu ?", "fr-FR", Path::new("out.wav"));
        assert!(script.contains("'fr-FR'"));
        assert!(script.contains("'Dors-tu ?'"));
        assert!(script.contains("out.wav"));
        assert!(script.contains("System.Speech"));
    }

    #[test]
    fn recording_tts_captures_calls() {
        let tts = RecordingTts::default();
        tts.speak("hi", "en-US", Path::new("x")).unwrap();
        assert_eq!(
            tts.spoken.borrow()[0],
            ("en-US".to_string(), "hi".to_string())
        );
    }
}
