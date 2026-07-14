//! presence-monitor — Rust port of the PowerShell presence monitor.
//!
//! Pings a phone to detect presence, verifies transitions with a microphone
//! recording, greets on arrival via TTS, and enters a panic announcement if
//! no spoken response is heard within the timeout.

mod config;
mod mic;
mod monitor;
mod ping;
mod state;
mod tts;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;

use config::Config;
use mic::{FfmpegMic, MicRecorder};
use monitor::{process_cycle, FileLogger, Logger, Paths};
use ping::SystemPing;
use state::PresenceState;
use tts::{TtsEngine, WindowsSapiTts};

#[derive(Parser)]
#[command(
    name = "presence-monitor",
    version,
    about = "Ping-based presence monitor with mic verification, TTS greeting, and panic mode."
)]
struct Cli {
    /// Path to the JSON config (default: config.json next to the binary/cwd).
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Continuously monitor presence (default). Use --once for a single cycle.
    Run {
        /// Run a single presence cycle and exit.
        #[arg(long)]
        once: bool,
    },
    /// Validate config and report which backends are available. No hardware.
    SelfCheck,
    /// Record a short clip and speak the greeting (requires mic + TTS).
    SelfTest,
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

/// Locate the config file: explicit flag, else `config.json` in cwd, else
/// next to the executable.
fn resolve_config_path(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    let cwd = PathBuf::from("config.json");
    if cwd.exists() {
        return Ok(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let side = dir.join("config.json");
            if side.exists() {
                return Ok(side);
            }
        }
    }
    Ok(cwd)
}

/// The project root used for `tools/`, `logs/`, and `state.json` — the
/// directory containing the config file.
fn root_for(config_path: &PathBuf) -> PathBuf {
    config_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn real_main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = resolve_config_path(cli.config.clone())?;

    match cli.command {
        Some(Command::SelfCheck) => cmd_self_check(&config_path),
        Some(Command::SelfTest) => cmd_self_test(&config_path),
        Some(Command::Run { once }) => cmd_run(&config_path, once),
        None => cmd_run(&config_path, false),
    }
}

fn load_config(config_path: &PathBuf) -> Result<Config> {
    Config::load(config_path)
        .with_context(|| format!("loading config from {}", config_path.display()))
}

fn cmd_self_check(config_path: &PathBuf) -> Result<()> {
    println!("presence-monitor self-check");
    println!("  config path : {}", config_path.display());

    let cfg = match load_config(config_path) {
        Ok(c) => {
            println!("  config      : OK (parsed)");
            c
        }
        Err(e) => {
            println!("  config      : FAILED — {e:#}");
            std::process::exit(1);
        }
    };

    let problems = cfg.validate();
    if problems.is_empty() {
        println!("  validation  : OK");
    } else {
        println!("  validation  : {} problem(s)", problems.len());
        for p in &problems {
            println!("    - {p}");
        }
    }

    println!(
        "  device      : target={} interval={}s timeout={}ms",
        cfg.device.target, cfg.device.check_interval_s, cfg.device.ping_timeout_ms
    );
    println!(
        "  mic         : verify={}s threshold={}dB chunk={}s response_timeout={}s",
        cfg.mic.verify_seconds,
        cfg.mic.speech_threshold_db,
        cfg.mic.chunk_seconds,
        cfg.mic.response_timeout_s
    );

    // Backend availability — no hardware is exercised, only presence checks.
    let root = root_for(config_path);
    let ffmpeg = FfmpegMic::discover(&root);
    println!(
        "  ping backend: system `ping` ({})",
        if cfg!(windows) {
            "windows -n/-w"
        } else {
            "unix -c/-W"
        }
    );
    println!(
        "  mic backend : ffmpeg {}",
        if ffmpeg.available() {
            "available"
        } else {
            "NOT FOUND (place ffmpeg under tools/ or on PATH)"
        }
    );
    let tts = WindowsSapiTts;
    println!(
        "  tts backend : Windows SAPI {}",
        if tts.available() {
            "available"
        } else {
            "NOT available (needs Windows + PowerShell)"
        }
    );

    if problems.is_empty() {
        println!("self-check: OK");
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn cmd_self_test(config_path: &PathBuf) -> Result<()> {
    let cfg = load_config(config_path)?;
    let root = root_for(config_path);
    let paths = Paths::resolve(&root, &cfg);
    paths.ensure_dirs()?;
    let logger = FileLogger::new(paths.main_log.clone());

    logger.log("SELFTEST start");
    let mic = FfmpegMic::discover(&root);
    if !mic.available() {
        anyhow::bail!("ffmpeg not available — cannot run self-test");
    }
    let device = mic.default_device()?;
    logger.log(&format!("Mikrofon: {device}"));
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let wav = paths.audio_dir.join(format!("verify_{stamp}.wav"));
    let res = mic.record_clip(3, &wav)?;
    logger.log(&format!(
        "selftest clip: {} max={}dB",
        wav.display(),
        res.max_db
    ));

    let tts = WindowsSapiTts;
    let greeting_wav = paths.audio_dir.join(format!("tts_{stamp}.wav"));
    tts.speak(
        &cfg.tts.greeting_text,
        &cfg.tts.greeting_language,
        &greeting_wav,
    )?;
    logger.log(&format!("selftest greeting: {}", greeting_wav.display()));
    logger.log("SELFTEST ende");
    Ok(())
}

fn cmd_run(config_path: &PathBuf, once: bool) -> Result<()> {
    let cfg = load_config(config_path)?;
    let root = root_for(config_path);
    let paths = Paths::resolve(&root, &cfg);
    paths.ensure_dirs()?;
    let logger = FileLogger::new(paths.main_log.clone());

    let probe = SystemPing;
    let mic = FfmpegMic::discover(&root);
    let tts = WindowsSapiTts;
    let mut state = PresenceState::load(&paths.state_file);

    if !mic.available() {
        logger.log("WARN: ffmpeg not found — mic verification will error on transitions");
    }

    let run_cycle = |state: &mut PresenceState| -> Result<()> {
        process_cycle(&probe, &mic, &tts, &cfg, state, &paths, &logger)
    };

    if once {
        run_cycle(&mut state)?;
        return Ok(());
    }

    loop {
        if let Err(e) = run_cycle(&mut state) {
            logger.log(&format!("cycle error: {e:#}"));
        }
        std::thread::sleep(Duration::from_secs(cfg.device.check_interval_s));
    }
}
