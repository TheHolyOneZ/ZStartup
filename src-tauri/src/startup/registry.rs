use super::approved;
use super::resolve::{make_id, reg_err, resolve_command};
use crate::models::{EntryType, Scope, Source, StartupEntry};
use winreg::enums::*;
use winreg::{RegKey, RegValue};

const LEGACY_SUFFIX: &str = ":disabled";

struct RunKey {
    id: &'static str,
    machine: bool,
    path: &'static str,
    approved: Option<&'static str>,
    entry_type: EntryType,
}

impl RunKey {
    fn hive(&self) -> RegKey {
        RegKey::predef(if self.machine { HKEY_LOCAL_MACHINE } else { HKEY_CURRENT_USER })
    }

    fn legacy_path(&self) -> String {
        format!(r"{}\Disabled", self.path)
    }

    fn scope(&self) -> Scope {
        if self.machine { Scope::AllUsers } else { Scope::CurrentUser }
    }
}

fn run_keys() -> [RunKey; 6] {
    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const RUN_ONCE: &str = r"Software\Microsoft\Windows\CurrentVersion\RunOnce";
    const RUN_32: &str = r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run";
    const RUN_ONCE_32: &str = r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\RunOnce";
    [
        RunKey { id: "reg_user_run", machine: false, path: RUN, approved: Some(approved::RUN), entry_type: EntryType::Run },
        RunKey { id: "reg_user_runonce", machine: false, path: RUN_ONCE, approved: None, entry_type: EntryType::RunOnce },
        RunKey { id: "reg_system_run", machine: true, path: RUN, approved: Some(approved::RUN), entry_type: EntryType::Run },
        RunKey { id: "reg_system_runonce", machine: true, path: RUN_ONCE, approved: None, entry_type: EntryType::RunOnce },
        RunKey { id: "reg_system_run32", machine: true, path: RUN_32, approved: Some(approved::RUN32), entry_type: EntryType::Run },
        RunKey { id: "reg_system_runonce32", machine: true, path: RUN_ONCE_32, approved: None, entry_type: EntryType::RunOnce },
    ]
}

fn find(source_id: &str) -> Result<(RunKey, bool), String> {
    let (id, legacy) = match source_id.strip_suffix(LEGACY_SUFFIX) {
        Some(id) => (id, true),
        None => (source_id, false),
    };
    run_keys()
        .into_iter()
        .find(|k| k.id == id)
        .map(|k| (k, legacy))
        .ok_or_else(|| format!("Unknown source: {source_id}"))
}

fn make_entry(k: &RunKey, name: String, value: &RegValue, enabled: bool, legacy: bool) -> StartupEntry {
    let command = value.to_string();
    let r = resolve_command(&command);
    let source_id = if legacy { format!("{}{LEGACY_SUFFIX}", k.id) } else { k.id.to_string() };
    StartupEntry {
        id: make_id(&source_id, &name),
        key: name.clone(),
        name,
        command,
        exe_path: r.exe_path,
        args: r.args,
        source: Source::Registry,
        source_id,
        scope: k.scope(),
        enabled,
        file_exists: r.exists,
        entry_type: k.entry_type.clone(),
    }
}

pub fn read_registry_entries() -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    for k in run_keys() {
        let hive = k.hive();
        if let Ok(key) = hive.open_subkey(k.path) {
            for (name, value) in key.enum_values().filter_map(Result::ok) {
                if name.is_empty() && value.bytes.len() <= 2 {
                    continue;
                }
                let enabled = k
                    .approved
                    .is_none_or(|list| !approved::is_disabled(&hive, list, &name));
                entries.push(make_entry(&k, name, &value, enabled, false));
            }
        }
        if let Ok(key) = hive.open_subkey(k.legacy_path()) {
            for (name, value) in key.enum_values().filter_map(Result::ok) {
                entries.push(make_entry(&k, name, &value, false, true));
            }
        }
    }
    entries
}

fn move_value(hive: &RegKey, from: &str, to: &str, name: &str) -> Result<(), String> {
    let src = hive
        .open_subkey_with_flags(from, KEY_READ | KEY_SET_VALUE)
        .map_err(reg_err)?;
    let value = src.get_raw_value(name).map_err(reg_err)?;
    let (dst, _) = hive.create_subkey(to).map_err(reg_err)?;
    dst.set_raw_value(name, &value).map_err(reg_err)?;
    src.delete_value(name).map_err(reg_err)
}

fn drop_if_empty(hive: &RegKey, path: &str) {
    let empty = hive
        .open_subkey(path)
        .and_then(|k| k.query_info())
        .is_ok_and(|info| info.values == 0 && info.sub_keys == 0);
    if empty {
        let _ = hive.delete_subkey(path);
    }
}

pub fn set_registry_entry_enabled(source_id: &str, name: &str, enabled: bool) -> Result<(), String> {
    let (k, legacy) = find(source_id)?;
    let hive = k.hive();
    let legacy_path = k.legacy_path();

    if legacy {
        if !enabled {
            return Ok(());
        }
        move_value(&hive, &legacy_path, k.path, name)?;
        drop_if_empty(&hive, &legacy_path);
        return match k.approved {
            Some(list) => approved::set(&hive, list, name, true),
            None => Ok(()),
        };
    }

    match k.approved {
        Some(list) => approved::set(&hive, list, name, enabled),
        None if enabled => Ok(()),
        None => move_value(&hive, k.path, &legacy_path, name),
    }
}

pub fn delete_registry_entry(source_id: &str, name: &str) -> Result<(), String> {
    let (k, legacy) = find(source_id)?;
    let hive = k.hive();
    let path = if legacy { k.legacy_path() } else { k.path.to_string() };
    let key = hive
        .open_subkey_with_flags(&path, KEY_SET_VALUE)
        .map_err(reg_err)?;
    key.delete_value(name).map_err(reg_err)?;
    if legacy {
        drop_if_empty(&hive, &path);
    }
    if let Some(list) = k.approved {
        approved::remove(&hive, list, name);
    }
    Ok(())
}

pub fn add_registry_entry(name: &str, command: &str, scope: &str) -> Result<(), String> {
    let k = run_keys()
        .into_iter()
        .find(|k| k.id == if scope == "all_users" { "reg_system_run" } else { "reg_user_run" })
        .ok_or("Unknown scope")?;
    let hive = k.hive();
    let (key, _) = hive.create_subkey(k.path).map_err(reg_err)?;
    let bytes = command
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect();
    let vtype = if command.contains('%') { REG_EXPAND_SZ } else { REG_SZ };
    key.set_raw_value(name, &RegValue { bytes, vtype }).map_err(reg_err)?;
    if let Some(list) = k.approved {
        approved::remove(&hive, list, name);
    }
    Ok(())
}
