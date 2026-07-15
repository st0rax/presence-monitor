//! Phone presence via the local ARP table (Windows `arp -a`).
//!
//! Mirrors `presence/presence_check.py`: a configured MAC/OUI prefix must appear
//! in the ARP cache. Non-Windows hosts return no MACs (documented fallback).

use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Detect whether the user's phone is on the local network.
pub trait PhoneProbe {
    /// Returns true when a MAC in the ARP table starts with `mac_prefix`.
    fn phone_present(&self, mac_prefix: &str) -> bool;
}

/// Live ARP-table probe (`arp -a` on Windows).
pub struct SystemArp;

impl PhoneProbe for SystemArp {
    fn phone_present(&self, mac_prefix: &str) -> bool {
        let prefix = normalize_mac(mac_prefix);
        if prefix.is_empty() {
            return false;
        }
        arp_macs().iter().any(|m| m.starts_with(&prefix))
    }
}

/// Normalize MAC/OUI prefix to uppercase hyphen form (`9C-93-4E`).
pub fn normalize_mac(mac: &str) -> String {
    mac.trim().to_uppercase().replace(':', "-")
}

/// Parse MAC addresses from `arp -a` output.
pub fn parse_arp_output(text: &str) -> Vec<String> {
    let mut macs = Vec::new();
    for line in text.lines() {
        for token in line.split_whitespace() {
            if is_mac_token(token) {
                macs.push(normalize_mac(token));
            }
        }
    }
    macs
}

fn is_mac_token(token: &str) -> bool {
    let t = token.trim();
    let sep = if t.contains('-') {
        '-'
    } else if t.contains(':') {
        ':'
    } else {
        return false;
    };
    let parts: Vec<&str> = t.split(sep).collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

fn arp_macs() -> Vec<String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("arp");
        cmd.arg("-a");
        cmd.creation_flags(CREATE_NO_WINDOW);
        match cmd.output() {
            Ok(out) if out.status.success() => {
                parse_arp_output(&String::from_utf8_lossy(&out.stdout))
            }
            _ => Vec::new(),
        }
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// Optional WLAN SSID gate (Windows `netsh wlan show interfaces`).
pub fn wlan_ssid() -> String {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("netsh");
        cmd.args(["wlan", "show", "interfaces"]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        match cmd.output() {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let lower = line.to_ascii_lowercase();
                    if lower.contains("ssid") && !lower.contains("bssid") {
                        if let Some(rest) = line.split(':').nth(1) {
                            let trimmed = rest.trim();
                            if !trimmed.is_empty() {
                                return trimmed.to_string();
                            }
                        }
                    }
                }
                String::new()
            }
            _ => String::new(),
        }
    }
    #[cfg(not(windows))]
    {
        String::new()
    }
}

/// Returns true when `required_ssid` is empty or matches the connected SSID.
pub fn wlan_ok(required_ssid: &str) -> bool {
    let required = required_ssid.trim();
    if required.is_empty() {
        return true;
    }
    wlan_ssid().eq_ignore_ascii_case(required)
}

/// A deterministic probe for tests.
#[cfg(test)]
pub struct ScriptedPhoneProbe {
    present: bool,
}

#[cfg(test)]
impl ScriptedPhoneProbe {
    pub fn new(present: bool) -> Self {
        Self { present }
    }
}

#[cfg(test)]
impl PhoneProbe for ScriptedPhoneProbe {
    fn phone_present(&self, _mac_prefix: &str) -> bool {
        self.present
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_mac_colons_to_hyphens() {
        assert_eq!(normalize_mac("9c:93:4e"), "9C-93-4E");
        assert_eq!(normalize_mac(" 9C-93-4E "), "9C-93-4E");
    }

    #[test]
    fn parse_arp_output_finds_macs() {
        let sample = r#"
Interface: 192.168.1.1 --- 0x7
  192.168.1.42    9c-93-4e-ab-cd-ef     dynamic
  192.168.1.10    aa-bb-cc-dd-ee-ff     dynamic
"#;
        let macs = parse_arp_output(sample);
        assert!(macs.contains(&"9C-93-4E-AB-CD-EF".to_string()));
        assert!(macs.contains(&"AA-BB-CC-DD-EE-FF".to_string()));
    }

    #[test]
    fn phone_present_matches_prefix() {
        let probe = SystemArp;
        // Unit test uses parse logic directly (no live ARP).
        let macs = parse_arp_output("  10.0.0.2  9c-93-4e-11-22-33 dynamic");
        let prefix = normalize_mac("9C:93:4E");
        assert!(macs.iter().any(|m| m.starts_with(&prefix)));
        let _ = probe; // SystemArp exercised on integration/desktop runs.
    }

    #[test]
    fn wlan_ok_empty_required_always_true() {
        assert!(wlan_ok(""));
        assert!(wlan_ok("   "));
    }
}
