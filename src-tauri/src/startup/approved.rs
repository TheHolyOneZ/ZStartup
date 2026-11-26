use super::resolve::reg_err;
use std::time::{SystemTime, UNIX_EPOCH};
use winreg::enums::*;
use winreg::{RegKey, RegValue};

const BASE: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";

pub const RUN: &str = "Run";
pub const RUN32: &str = "Run32";
pub const STARTUP_FOLDER: &str = "StartupFolder";

pub fn is_disabled(hive: &RegKey, list: &str, name: &str) -> bool {
    hive.open_subkey(format!(r"{BASE}\{list}"))
        .and_then(|k| k.get_raw_value(name))
        .map(|v| v.bytes.first().is_some_and(|b| b & 1 == 1))
        .unwrap_or(false)
}

pub fn set(hive: &RegKey, list: &str, name: &str, enabled: bool) -> Result<(), String> {
    let (key, _) = hive
        .create_subkey(format!(r"{BASE}\{list}"))
        .map_err(reg_err)?;
    let stamp = if enabled { 0 } else { filetime_now() };
    let mut bytes = vec![if enabled { 0x02 } else { 0x03 }, 0, 0, 0];
    bytes.extend_from_slice(&stamp.to_le_bytes());
    key.set_raw_value(name, &RegValue { bytes, vtype: REG_BINARY })
        .map_err(reg_err)
}

pub fn remove(hive: &RegKey, list: &str, name: &str) {
    if let Ok(key) = hive.open_subkey_with_flags(format!(r"{BASE}\{list}"), KEY_SET_VALUE) {
        let _ = key.delete_value(name);
    }
}

fn filetime_now() -> u64 {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (d.as_secs() + 11_644_473_600) * 10_000_000 + u64::from(d.subsec_nanos() / 100)
}
