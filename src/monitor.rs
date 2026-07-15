//! Orchestration: the presence-check cycle and the arrival greeting flow.
//!
//! All hardware/network effects are injected as trait objects
//! ([`PresenceProbe`], [`MicRecorder`], [`TtsEngine`]) so the whole cycle can
//! be exercised in tests with fakes. This mirrors `Process-Cycle` from the
//! PowerShell original.

use crate::arp::PhoneProbe;
use crate::clock::{file_stamp, log_timestamp, now_utc, rfc3339_now};
use crate::config::Config;
use crate::mic::{MicLevelSampler, MicRecorder};
use crate::ping::PresenceProbe;
use crate::presence::check_presence;
use crate::state::{PresenceState, Transition};
use crate::tts::TtsEngine;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Resolved filesystem layout (mirrors the PowerShell paths).
pub struct Paths {
    pub _root: PathBuf,
    pub log_dir: PathBuf,
    pub audio_dir: PathBuf,
    pub transition_dir: PathBuf,
    pub state_file: PathBuf,
    pub main_log: PathBuf,
}

impl Paths {
    pub fn resolve(root: &Path, cfg: &Config) -> Self {
        let log_dir = root.join(&cfg.logging.log_dir);
        let audio_dir = root.join(&cfg.logging.audio_dir);
        let transition_dir = root.join(&cfg.logging.transition_dir);
        Paths {
            main_log: log_dir.join("presence.log"),
            state_file: root.join("state.json"),
            log_dir,
            audio_dir,
            transition_dir,
            _root: root.to_path_buf(),
        }
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        for d in [&self.log_dir, &self.audio_dir, &self.transition_dir] {
            std::fs::create_dir_all(d)?;
        }
        Ok(())
    }
}

/// Sink for the running text log. Real runs write to file + stdout; tests use
/// an in-memory collector.
pub trait Logger {
    fn log(&self, msg: &str);
}

/// File+stdout logger matching the original `Write-Log` timestamp format.
pub struct FileLogger {
    path: PathBuf,
}

impl FileLogger {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Logger for FileLogger {
    fn log(&self, msg: &str) {
        let ts = log_timestamp();
        let line = format!("{ts} {msg}");
        println!("{line}");
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(f, "{line}");
        }
    }
}

/// Outcome of the arrival (absent -> present) greeting flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrivalOutcome {
    /// A spoken response was detected within the timeout.
    Answered,
    /// No response — panic announcement was made.
    Panic,
}

/// Poll the microphone in `chunk_seconds` slices until a chunk exceeds the
/// speech threshold or the total timeout elapses. Recording itself consumes
/// real time (as in the original), so no extra sleeping is needed here.
pub fn wait_for_response<M: MicRecorder + ?Sized>(
    mic: &M,
    cfg: &Config,
    audio_dir: &Path,
    logger: &dyn Logger,
) -> Result<bool> {
    let chunk = cfg.mic.chunk_seconds.max(1);
    let mut elapsed = 0u32;
    while elapsed < cfg.mic.response_timeout_s {
        let stamp = file_stamp();
        let clip = audio_dir.join(format!("resp_{stamp}.wav"));
        let res = mic.record_clip(chunk, &clip)?;
        if res.max_db > cfg.mic.speech_threshold_db {
            logger.log(&format!(
                "antwort-erkannt (clip: {}, max={} dB)",
                clip.display(),
                res.max_db
            ));
            return Ok(true);
        }
        let _ = std::fs::remove_file(&clip);
        elapsed += chunk;
    }
    Ok(false)
}

/// Run the arrival flow: greeting TTS, cooldown, listen for a response, and
/// panic TTS if none arrives.
pub fn handle_arrival<M: MicRecorder + ?Sized, T: TtsEngine + ?Sized>(
    mic: &M,
    tts: &T,
    cfg: &Config,
    audio_dir: &Path,
    logger: &dyn Logger,
) -> Result<ArrivalOutcome> {
    let greeting_wav = audio_dir.join(format!("tts_{}.wav", file_stamp()));
    tts.speak(
        &cfg.tts.greeting_text,
        &cfg.tts.greeting_language,
        &greeting_wav,
    )?;
    logger.log(&format!("begruessung (TTS): {}", greeting_wav.display()));

    logger.log(&format!(
        "warte {}s bis Mikrofon-Echo der Begruessung abgeklungen ist",
        cfg.mic.response_cooldown_s
    ));
    if cfg.mic.response_cooldown_s > 0 {
        std::thread::sleep(Duration::from_secs(cfg.mic.response_cooldown_s));
    }

    let answered = wait_for_response(mic, cfg, audio_dir, logger)?;
    if answered {
        logger.log("antwort erkannt (Storax hat geantwortet)");
        Ok(ArrivalOutcome::Answered)
    } else {
        let panic_wav = audio_dir.join(format!("tts_{}.wav", file_stamp()));
        tts.speak(&cfg.tts.panic_text, &cfg.tts.panic_language, &panic_wav)?;
        logger.log(&format!("PANICMODE: {}", panic_wav.display()));
        Ok(ArrivalOutcome::Panic)
    }
}

/// Record a verification clip on a transition and persist its JSON metadata,
/// mirroring `Record-VerifyClip`.
fn record_verify_clip<M: MicRecorder + ?Sized>(
    mic: &M,
    cfg: &Config,
    paths: &Paths,
    logger: &dyn Logger,
) -> Result<()> {
    let stamp = file_stamp();
    let wav = paths.audio_dir.join(format!("verify_{stamp}.wav"));
    let res = mic.record_clip(cfg.mic.verify_seconds, &wav)?;
    let meta = serde_json::json!({
        "timestamp": rfc3339_now(),
        "type": "verify",
        "audioFile": wav.to_string_lossy(),
        "maxDb": (res.max_db * 100.0).round() / 100.0,
        "durationS": res.duration_s,
    });
    let json_path = paths.transition_dir.join(format!("verify_{stamp}.json"));
    std::fs::write(&json_path, serde_json::to_string_pretty(&meta)?)?;
    logger.log(&format!(
        "verify clip: {} max={}dB",
        wav.display(),
        (res.max_db * 100.0).round() / 100.0
    ));
    Ok(())
}

/// Die austauschbaren Aussenkanten eines Zyklus.
///
/// Als Struct statt acht Einzelparameter — aus zwei Gruenden, die dasselbe
/// Problem sind: `clippy::too_many_arguments` schlug bei 8/7 an (und liess die CI
/// rot), und der Ping war als einziger **kein** Seam. Er wurde in `process_cycle`
/// hart als `SystemPing` instanziiert, obwohl `phone`/`mic`/`tts` alle injiziert
/// sind — wodurch jeder Zyklus-Test echtes `ping.exe` ausschellte, waehrend das
/// vorhandene `ScriptedProbe` ungenutzt blieb. Jetzt haengt er am selben Haken.
pub struct Probes<'a, P: ?Sized, M: ?Sized, T: ?Sized> {
    pub phone: &'a P,
    pub mic_sampler: &'a M,
    pub mic: &'a M,
    pub tts: &'a T,
    pub ping: &'a dyn PresenceProbe,
}

/// Run one full presence cycle: ARP phone + mic RMS, update state, act on transitions.
pub fn process_cycle<P, M, T>(
    probes: &Probes<'_, P, M, T>,
    cfg: &Config,
    state: &mut PresenceState,
    paths: &Paths,
    logger: &dyn Logger,
) -> Result<()>
where
    P: PhoneProbe + ?Sized,
    M: MicLevelSampler + MicRecorder + ?Sized,
    T: TtsEngine + ?Sized,
{
    let (mic, tts) = (probes.mic, probes.tts);
    // Ping fliesst zusaetzlich zu ARP/Mic in das Urteil ein (Legacy-Target).
    let ping_target = if cfg.device.target.trim().is_empty() {
        "127.0.0.1".to_string()
    } else {
        cfg.device.target.clone()
    };
    let ping_present = probes
        .ping
        .is_present(&ping_target, cfg.device.ping_timeout_ms);
    let verdict = check_presence(
        probes.phone,
        probes.mic_sampler,
        &cfg.device.phone_mac_prefix,
        &cfg.device.wlan_ssid,
        cfg.mic.rms_threshold,
        1.0,
    );
    let present = verdict.present || ping_present;
    let tr = state.observe(present, now_utc());
    let state_now = if present { "present" } else { "absent" };
    // ping mitloggen: er geht ins Urteil ein, fehlte aber in der Zeile — ein nur
    // per Ping anwesender Rechner loggte "phone=false voice=false -> present".
    logger.log(&format!(
        "presence phone={} voice={} rms={:.4} ping={} -> {}",
        verdict.phone_present, verdict.voice_detected, verdict.mic_rms, ping_present, state_now
    ));

    match tr {
        Transition::Initial { .. } => {
            logger.log(&format!("initial state = {state_now} (kein Uebergang)"));
            state.save(&paths.state_file)?;
        }
        Transition::Unchanged { .. } => {}
        Transition::Changed { from, to } => {
            let from_s = if from { "present" } else { "absent" };
            let to_s = if to { "present" } else { "absent" };
            logger.log(&format!("UEBERGANG {from_s} -> {to_s}"));
            record_verify_clip(mic, cfg, paths, logger)?;
            if tr.is_arrival() {
                handle_arrival(mic, tts, cfg, &paths.audio_dir, logger)?;
            }
            state.save(&paths.state_file)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arp::ScriptedPhoneProbe;
    use crate::mic::FakeMic;
    use crate::ping::ScriptedProbe;
    use crate::tts::RecordingTts;
    use std::cell::RefCell;

    struct MemLogger {
        lines: RefCell<Vec<String>>,
    }
    impl MemLogger {
        fn new() -> Self {
            Self {
                lines: RefCell::new(Vec::new()),
            }
        }
        fn joined(&self) -> String {
            self.lines.borrow().join("\n")
        }
    }
    impl Logger for MemLogger {
        fn log(&self, msg: &str) {
            self.lines.borrow_mut().push(msg.to_string());
        }
    }

    fn test_config() -> Config {
        let json = r#"{
          "device": { "target": "host", "ping_timeout_ms": 100, "check_interval_s": 1 },
          "mic": { "verify_seconds": 1, "response_timeout_s": 6, "response_cooldown_s": 0, "speech_threshold_db": -40.0, "chunk_seconds": 2 },
          "tts": { "greeting_language": "fr-FR", "panic_language": "de-DE", "greeting_text": "hi", "panic_text": "panic" },
          "logging": { "log_dir": "logs", "audio_dir": "logs/audio", "transition_dir": "logs/transitions" }
        }"#;
        Config::from_json_str(json).unwrap()
    }

    fn tmp_dir(tag: &str) -> PathBuf {
        let mut d = std::env::temp_dir();
        d.push(format!(
            "presmon_test_{}_{}",
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn wait_for_response_detects_loud_chunk() {
        let cfg = test_config();
        // 3 quiet chunks then a loud one; threshold -40.
        let mic = FakeMic::new(vec![-60.0, -55.0, -10.0]);
        let dir = tmp_dir("wait_yes");
        let logger = MemLogger::new();
        let answered = wait_for_response(&mic, &cfg, &dir, &logger).unwrap();
        assert!(answered);
    }

    #[test]
    fn wait_for_response_times_out_when_quiet() {
        let cfg = test_config(); // timeout 6, chunk 2 -> 3 chunks
        let mic = FakeMic::new(vec![-60.0, -60.0, -60.0]);
        let dir = tmp_dir("wait_no");
        let logger = MemLogger::new();
        let answered = wait_for_response(&mic, &cfg, &dir, &logger).unwrap();
        assert!(!answered);
    }

    #[test]
    fn arrival_answered_skips_panic() {
        let cfg = test_config();
        let mic = FakeMic::new(vec![-5.0]); // immediately loud
        let tts = RecordingTts::default();
        let dir = tmp_dir("arr_yes");
        let logger = MemLogger::new();
        let outcome = handle_arrival(&mic, &tts, &cfg, &dir, &logger).unwrap();
        assert_eq!(outcome, ArrivalOutcome::Answered);
        // Only the greeting was spoken, no panic.
        let spoken = tts.spoken.borrow();
        assert_eq!(spoken.len(), 1);
        assert_eq!(spoken[0].0, "fr-FR");
    }

    #[test]
    fn arrival_silence_triggers_panic() {
        let cfg = test_config();
        let mic = FakeMic::new(vec![-60.0, -60.0, -60.0]);
        let tts = RecordingTts::default();
        let dir = tmp_dir("arr_no");
        let logger = MemLogger::new();
        let outcome = handle_arrival(&mic, &tts, &cfg, &dir, &logger).unwrap();
        assert_eq!(outcome, ArrivalOutcome::Panic);
        let spoken = tts.spoken.borrow();
        assert_eq!(spoken.len(), 2);
        assert_eq!(spoken[1].0, "de-DE"); // panic language
    }

    #[test]
    fn cycle_initial_then_arrival_flow() {
        let cfg = test_config();
        let dir = tmp_dir("cycle");
        let paths = Paths::resolve(&dir, &cfg);
        paths.ensure_dirs().unwrap();
        let logger = MemLogger::new();
        let tts = RecordingTts::default();

        // Ping deterministisch abwesend (leere Sequenz => immer false): vorher
        // shellte dieser Test bei jedem Lauf echtes ping.exe aus — und haette gegen
        // das 127.0.0.1-Fallback-Target immer "present" gemeldet.
        let ping = ScriptedProbe::new(vec![]);

        // First cycle absent -> initial (no verify, no tts).
        let phone = ScriptedPhoneProbe::new(false);
        let mic = FakeMic::with_rms(vec![], vec![0.001]);
        let mut state = PresenceState::new();
        process_cycle(
            &Probes {
                phone: &phone,
                mic_sampler: &mic,
                mic: &mic,
                tts: &tts,
                ping: &ping,
            },
            &cfg,
            &mut state,
            &paths,
            &logger,
        )
        .unwrap();
        assert_eq!(state.present, Some(false));
        assert!(tts.spoken.borrow().is_empty());

        // Next cycle present (phone) -> arrival: verify + greeting + loud resp.
        let phone = ScriptedPhoneProbe::new(true);
        let mic = FakeMic::with_rms(vec![-30.0 /*verify*/, -5.0 /*resp loud*/], vec![0.001]);
        process_cycle(
            &Probes {
                phone: &phone,
                mic_sampler: &mic,
                mic: &mic,
                tts: &tts,
                ping: &ping,
            },
            &cfg,
            &mut state,
            &paths,
            &logger,
        )
        .unwrap();
        assert_eq!(state.present, Some(true));
        assert_eq!(tts.spoken.borrow().len(), 1); // greeting only, answered
        assert!(logger.joined().contains("UEBERGANG absent -> present"));
        // Der Ping steht jetzt in der Zeile — vorher log "phone=.. voice=.." ohne ihn.
        assert!(logger.joined().contains("ping=false"));

        // A verify JSON was written.
        let n_json = std::fs::read_dir(&paths.transition_dir).unwrap().count();
        assert_eq!(n_json, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
