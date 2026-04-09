use crate::models::{Source, StartupEntry};
use crate::startup::{folder, registry};
use std::path::Path;
use tauri::command;

#[command]
pub fn is_admin() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            KEY_WRITE,
        )
        .is_ok()
}

#[command]
pub fn restart_as_admin() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe.to_string_lossy();
    std::process::Command::new("powershell")
        .args([
            "-WindowStyle", "Hidden",
            "-Command",
            &format!("Start-Process '{}' -Verb RunAs", exe_str),
        ])
        .spawn()
        .map_err(|e| e.to_string())?;
    std::process::exit(0);
}

#[command]
pub fn get_startup_entries() -> Vec<StartupEntry> {
    let mut entries = registry::read_registry_entries();
    entries.extend(folder::read_folder_entries());
    entries.sort_by(|a, b| {
        b.enabled
            .cmp(&a.enabled)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}

#[command]
pub fn toggle_entry(
    source_id: String,
    name: String,
    source: Source,
    enabled: bool,
) -> Result<(), String> {
    match source {
        Source::Registry => registry::set_registry_entry_enabled(&source_id, &name, enabled),
        Source::Folder => folder::set_folder_entry_enabled(&source_id, &name, enabled),
    }
}

#[command]
pub fn delete_entry(
    source_id: String,
    name: String,
    source: Source,
    enabled: bool,
) -> Result<(), String> {
    match source {
        Source::Registry => registry::delete_registry_entry(&source_id, &name, enabled),
        Source::Folder => folder::delete_folder_entry(&source_id, &name, enabled),
    }
}

#[command]
pub fn add_entry(name: String, command: String, scope: String) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Name cannot be empty".into());
    }
    if command.trim().is_empty() {
        return Err("Command cannot be empty".into());
    }
    registry::add_registry_entry(&name, &command, &scope)
}

#[command]
pub fn open_file_location(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let folder = if p.is_file() {
        p.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path.clone())
    } else {
        path.clone()
    };

    std::process::Command::new("explorer")
        .arg(&folder)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
