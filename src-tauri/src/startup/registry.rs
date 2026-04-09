use crate::models::{EntryType, Scope, Source, StartupEntry};
use sha2::{Digest, Sha256};
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

fn make_id(source_id: &str, name: &str) -> String {
    let mut h = Sha256::new();
    h.update(source_id);
    h.update(name);
    hex::encode(h.finalize())[..16].to_string()
}

fn parse_exe(command: &str) -> (String, String) {
    let cmd = command.trim();
    
    if cmd.starts_with('"') {
        if let Some(end) = cmd[1..].find('"') {
            let exe = cmd[1..end + 1].to_string();
            let args = cmd[end + 2..].trim().to_string();
            return (exe, args);
        }
    }
    
    if let Some(sp) = cmd.find(' ') {
        (cmd[..sp].to_string(), cmd[sp + 1..].trim().to_string())
    } else {
        (cmd.to_string(), String::new())
    }
}

fn read_run_key(
    hive: RegKey,
    key_path: &str,
    disabled_key_path: &str,
    scope: Scope,
    entry_type: EntryType,
    source_id: &str,
    entries: &mut Vec<StartupEntry>,
) {
    
    if let Ok(key) = hive.open_subkey(key_path) {
        for value in key.enum_values().filter_map(|v| v.ok()) {
            let name = value.0;
            let command: String = value.1.to_string();
            let (exe_path, args) = parse_exe(&command);
            let file_exists = Path::new(&exe_path).exists();
            entries.push(StartupEntry {
                id: make_id(source_id, &name),
                name,
                command,
                exe_path,
                args,
                source: Source::Registry,
                source_id: source_id.to_string(),
                scope: scope.clone(),
                enabled: true,
                file_exists,
                entry_type: entry_type.clone(),
            });
        }
    }

    
    let disabled_path = format!("{}\\Disabled", key_path);
    if let Ok(dis_key) = hive.open_subkey(&disabled_path) {
        for value in dis_key.enum_values().filter_map(|v| v.ok()) {
            let name = value.0;
            let command: String = value.1.to_string();
            let (exe_path, args) = parse_exe(&command);
            let file_exists = Path::new(&exe_path).exists();
            let sid = format!("{}_disabled", source_id);
            entries.push(StartupEntry {
                id: make_id(&sid, &name),
                name,
                command,
                exe_path,
                args,
                source: Source::Registry,
                source_id: source_id.to_string(),
                scope: scope.clone(),
                enabled: false,
                file_exists,
                entry_type: entry_type.clone(),
            });
        }
    }
    let _ = disabled_key_path;
}

pub fn read_registry_entries() -> Vec<StartupEntry> {
    let mut entries = Vec::new();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    
    read_run_key(
        RegKey::predef(HKEY_CURRENT_USER),
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        "",
        Scope::CurrentUser,
        EntryType::Run,
        "reg_user_run",
        &mut entries,
    );

    
    read_run_key(
        RegKey::predef(HKEY_CURRENT_USER),
        r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
        "",
        Scope::CurrentUser,
        EntryType::RunOnce,
        "reg_user_runonce",
        &mut entries,
    );

    
    read_run_key(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        "",
        Scope::AllUsers,
        EntryType::Run,
        "reg_system_run",
        &mut entries,
    );

    
    read_run_key(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
        "",
        Scope::AllUsers,
        EntryType::RunOnce,
        "reg_system_runonce",
        &mut entries,
    );

    let _ = hkcu;
    let _ = hklm;
    entries
}

pub fn set_registry_entry_enabled(
    source_id: &str,
    name: &str,
    enabled: bool,
) -> Result<(), String> {
    let (hive_predef, key_path) = match source_id {
        "reg_user_run" => (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
        ),
        "reg_user_runonce" => (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
        ),
        "reg_system_run" => (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
        ),
        "reg_system_runonce" => (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
        ),
        _ => return Err(format!("Unknown source_id: {}", source_id)),
    };

    let hive = RegKey::predef(hive_predef);
    let disabled_path = format!("{}\\Disabled", key_path);

    if enabled {
        
        let dis_key = hive
            .open_subkey_with_flags(&disabled_path, KEY_READ | KEY_WRITE)
            .map_err(|e| e.to_string())?;
        let value: String = dis_key.get_value(name).map_err(|e| e.to_string())?;
        dis_key.delete_value(name).map_err(|e| e.to_string())?;
        let run_key = hive
            .open_subkey_with_flags(key_path, KEY_WRITE)
            .map_err(|e| e.to_string())?;
        run_key.set_value(name, &value).map_err(|e| e.to_string())?;
    } else {
        
        let run_key = hive
            .open_subkey_with_flags(key_path, KEY_READ | KEY_WRITE)
            .map_err(|e| e.to_string())?;
        let value: String = run_key.get_value(name).map_err(|e| e.to_string())?;
        run_key.delete_value(name).map_err(|e| e.to_string())?;
        let (dis_key, _) = hive
            .create_subkey(&disabled_path)
            .map_err(|e| e.to_string())?;
        dis_key
            .set_value(name, &value)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn delete_registry_entry(source_id: &str, name: &str, enabled: bool) -> Result<(), String> {
    let (hive_predef, key_path) = match source_id {
        "reg_user_run" => (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
        ),
        "reg_user_runonce" => (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
        ),
        "reg_system_run" => (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
        ),
        "reg_system_runonce" => (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
        ),
        _ => return Err(format!("Unknown source_id: {}", source_id)),
    };

    let hive = RegKey::predef(hive_predef);
    let target_path = if enabled {
        key_path.to_string()
    } else {
        format!("{}\\Disabled", key_path)
    };
    let key = hive
        .open_subkey_with_flags(&target_path, KEY_WRITE)
        .map_err(|e| e.to_string())?;
    key.delete_value(name).map_err(|e| e.to_string())
}

pub fn add_registry_entry(name: &str, command: &str, scope: &str) -> Result<(), String> {
    let (hive_predef, key_path) = match scope {
        "all_users" => (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
        ),
        _ => (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
        ),
    };
    let hive = RegKey::predef(hive_predef);
    let key = hive
        .open_subkey_with_flags(key_path, KEY_WRITE)
        .map_err(|e| e.to_string())?;
    key.set_value(name, &command.to_string())
        .map_err(|e| e.to_string())
}
