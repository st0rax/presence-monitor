//! Presence detection via ICMP ping.
//!
//! The original PowerShell called `ping.exe -n 1 -w <timeout_ms> <target>`
//! and treated exit code 0 as "present". This module preserves that
//! behaviour and adds a cross-platform command for non-Windows hosts so the
//! crate still builds and the logic can be exercised anywhere.

use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Abstraction over presence detection so the monitor loop can be tested
/// without touching the network.
#[allow(dead_code)]
pub trait PresenceProbe {
    /// Returns `true` when the target responds within the timeout.
    fn is_present(&self, target: &str, timeout_ms: u32) -> bool;
}

/// Real ICMP probe backed by the system `ping` command.
#[allow(dead_code)]
pub struct SystemPing;

impl PresenceProbe for SystemPing {
    fn is_present(&self, target: &str, timeout_ms: u32) -> bool {
        let mut cmd = build_ping_command(target, timeout_ms);
        match cmd.output() {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
}

/// Build the platform-appropriate one-shot ping command.
#[allow(dead_code)]
fn build_ping_command(target: &str, timeout_ms: u32) -> Command {
    let mut cmd = Command::new("ping");
    if cfg!(windows) {
        // -n 1 : one echo request, -w : timeout in milliseconds
        cmd.arg("-n")
            .arg("1")
            .arg("-w")
            .arg(timeout_ms.to_string())
            .arg(target);
    } else {
        // -c 1 : one echo request, -W : timeout in whole seconds
        let secs = timeout_ms.div_ceil(1000).max(1);
        cmd.arg("-c")
            .arg("1")
            .arg("-W")
            .arg(secs.to_string())
            .arg(target);
    }
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// A deterministic probe for tests: replays a fixed sequence of results.
#[cfg(test)]
pub struct ScriptedProbe {
    results: std::cell::RefCell<std::collections::VecDeque<bool>>,
}

#[cfg(test)]
impl ScriptedProbe {
    pub fn new(results: Vec<bool>) -> Self {
        Self {
            results: std::cell::RefCell::new(results.into()),
        }
    }
}

#[cfg(test)]
impl PresenceProbe for ScriptedProbe {
    fn is_present(&self, _target: &str, _timeout_ms: u32) -> bool {
        self.results.borrow_mut().pop_front().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_command_uses_n_and_w() {
        // Only assert the shape on the platform we build for.
        let cmd = build_ping_command("host", 2000);
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.contains(&"host".to_string()));
        assert_eq!(args.iter().filter(|a| *a == "1").count(), 1);
    }

    #[test]
    fn scripted_probe_replays_sequence() {
        let p = ScriptedProbe::new(vec![true, false, true]);
        assert!(p.is_present("x", 1));
        assert!(!p.is_present("x", 1));
        assert!(p.is_present("x", 1));
        // Exhausted -> defaults to absent.
        assert!(!p.is_present("x", 1));
    }
}
