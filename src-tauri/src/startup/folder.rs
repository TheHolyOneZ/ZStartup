use crate::models::{EntryType, Scope, Source, StartupEntry};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

fn make_id(source_id: &str, name: &str) -> String {
    let mut h = Sha256::new();
    h.update(source_id);
    h.update(name);
    hex::encode(h.finalize())[..16].to_string()
}

fn parse_lnk_target(lnk_path: &Path) -> Option<String> {
    let data = fs::read(lnk_path).ok()?;

    
    if data.len() < 76 || &data[0..4] != &[0x4C, 0x00, 0x00, 0x00] {
        return None;
    }

    
    let link_flags = u32::from_le_bytes(data[0x14..0x18].try_into().ok()?);
    let has_link_target_id_list = link_flags & 0x01 != 0;
    let has_link_info = link_flags & 0x02 != 0;

    let mut offset: usize = 76; 

    
    if has_link_target_id_list {
        if offset + 2 > data.len() {
            return None;
        }
        let id_list_size = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?) as usize;
        offset += 2 + id_list_size;
    }

    
    if has_link_info {
        if offset + 4 > data.len() {
            return None;
        }
        let link_info_size =
            u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
        if offset + link_info_size > data.len() {
            return None;
        }

        
        if offset + 0x14 > data.len() {
            return None;
        }
        let local_path_offset =
            u32::from_le_bytes(data[offset + 0x10..offset + 0x14].try_into().ok()?) as usize;

        if local_path_offset > 0 {
            let abs_offset = offset + local_path_offset;
            if abs_offset < data.len() {
                let end = data[abs_offset..]
                    .iter()
                    .position(|&b| b == 0)
                    .map(|p| abs_offset + p)
                    .unwrap_or(data.len());
                if let Ok(path_str) = std::str::from_utf8(&data[abs_offset..end]) {
                    if !path_str.is_empty() {
                        return Some(path_str.to_string());
                    }
                }
            }
        }
    }

    None
}

fn read_startup_folder(folder: &Path, scope: Scope, source_id: &str) -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    let disabled_folder = folder.join("Disabled");

    let read_from = |dir: &Path, enabled: bool, entries: &mut Vec<StartupEntry>| {
        let Ok(read_dir) = fs::read_dir(dir) else { return };
        for entry in read_dir.filter_map(|e| e.ok()) {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext.eq_ignore_ascii_case("lnk") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    continue;
                }

                let target = parse_lnk_target(&path).unwrap_or_default();
                let file_exists = if target.is_empty() {
                    false
                } else {
                    Path::new(&target).exists()
                };

                let sid = if enabled {
                    source_id.to_string()
                } else {
                    format!("{}_disabled", source_id)
                };

                entries.push(StartupEntry {
                    id: make_id(&sid, &name),
                    name,
                    command: target.clone(),
                    exe_path: target,
                    args: String::new(),
                    source: Source::Folder,
                    source_id: source_id.to_string(),
                    scope: scope.clone(),
                    enabled,
                    file_exists,
                    entry_type: EntryType::Run,
                });
            }
        }
    };

    read_from(folder, true, &mut entries);
    read_from(&disabled_folder, false, &mut entries);
    entries
}

pub fn read_folder_entries() -> Vec<StartupEntry> {
    let mut entries = Vec::new();

    
    if let Ok(appdata) = std::env::var("APPDATA") {
        let user_startup = PathBuf::from(appdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        entries.extend(read_startup_folder(&user_startup, Scope::CurrentUser, "folder_user"));
    }

    
    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        let system_startup = PathBuf::from(programdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        entries.extend(read_startup_folder(&system_startup, Scope::AllUsers, "folder_system"));
    }

    entries
}

pub fn set_folder_entry_enabled(
    source_id: &str,
    name: &str,
    enabled: bool,
) -> Result<(), String> {
    let folder = get_folder_path(source_id)?;
    let disabled_folder = folder.join("Disabled");
    let lnk_name = format!("{}.lnk", name);

    if enabled {
        let from = disabled_folder.join(&lnk_name);
        let to = folder.join(&lnk_name);
        fs::rename(&from, &to).map_err(|e| e.to_string())
    } else {
        fs::create_dir_all(&disabled_folder).map_err(|e| e.to_string())?;
        let from = folder.join(&lnk_name);
        let to = disabled_folder.join(&lnk_name);
        fs::rename(&from, &to).map_err(|e| e.to_string())
    }
}

pub fn delete_folder_entry(source_id: &str, name: &str, enabled: bool) -> Result<(), String> {
    let folder = get_folder_path(source_id)?;
    let lnk_name = format!("{}.lnk", name);
    let path = if enabled {
        folder.join(&lnk_name)
    } else {
        folder.join("Disabled").join(&lnk_name)
    };
    fs::remove_file(&path).map_err(|e| e.to_string())
}

fn get_folder_path(source_id: &str) -> Result<PathBuf, String> {
    match source_id {
        "folder_user" => {
            let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
            Ok(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
        }
        "folder_system" => {
            let programdata = std::env::var("PROGRAMDATA").map_err(|e| e.to_string())?;
            Ok(PathBuf::from(programdata)
                .join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
        }
        _ => Err(format!("Unknown folder source_id: {}", source_id)),
    }
}
