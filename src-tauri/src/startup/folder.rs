use super::approved;
use super::resolve::{expand_env, make_id};
use crate::models::{EntryType, Scope, Source, StartupEntry};
use std::fs;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::RegKey;

const LEGACY_SUFFIX: &str = ":disabled";
const STARTUP_SUBPATH: &str = r"Microsoft\Windows\Start Menu\Programs\Startup";

struct Location {
    id: &'static str,
    machine: bool,
    dir: PathBuf,
}

impl Location {
    fn hive(&self) -> RegKey {
        RegKey::predef(if self.machine { HKEY_LOCAL_MACHINE } else { HKEY_CURRENT_USER })
    }

    fn legacy_dir(&self) -> PathBuf {
        self.dir.join("Disabled")
    }
}

fn locations() -> Vec<Location> {
    let mut out = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        out.push(Location {
            id: "folder_user",
            machine: false,
            dir: PathBuf::from(appdata).join(STARTUP_SUBPATH),
        });
    }
    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        out.push(Location {
            id: "folder_system",
            machine: true,
            dir: PathBuf::from(programdata).join(STARTUP_SUBPATH),
        });
    }
    out
}

fn find(source_id: &str) -> Result<(Location, bool), String> {
    let (id, legacy) = match source_id.strip_suffix(LEGACY_SUFFIX) {
        Some(id) => (id, true),
        None => (source_id, false),
    };
    locations()
        .into_iter()
        .find(|l| l.id == id)
        .map(|l| (l, legacy))
        .ok_or_else(|| format!("Unknown startup folder: {source_id}"))
}

fn u16_at(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes(d.get(o..o + 2)?.try_into().ok()?))
}

fn u32_at(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?))
}

fn cstr_at(d: &[u8], o: usize) -> Option<String> {
    let bytes = d.get(o..)?;
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let s = String::from_utf8_lossy(&bytes[..end]).into_owned();
    (o > 0 && !s.is_empty()).then_some(s)
}

fn wstr_from(bytes: &[u8]) -> String {
    let units: Vec<u16> = (0..bytes.len() / 2)
        .map(|i| u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]))
        .take_while(|&u| u != 0)
        .collect();
    String::from_utf16_lossy(&units)
}

fn wstr_at(d: &[u8], o: usize) -> Option<String> {
    let s = wstr_from(d.get(o..)?);
    (o > 0 && !s.is_empty()).then_some(s)
}

fn join_path(base: &str, suffix: &str) -> String {
    if suffix.is_empty() || base.ends_with('\\') {
        format!("{base}{suffix}")
    } else {
        format!(r"{base}\{suffix}")
    }
}

fn link_info_path(info: &[u8]) -> Option<String> {
    let header_size = u32_at(info, 4)? as usize;
    let flags = u32_at(info, 8)?;
    let unicode = header_size >= 0x24;
    let pick = |ansi_field: usize, wide_field: usize| -> Option<String> {
        let wide = if unicode {
            u32_at(info, wide_field).and_then(|o| wstr_at(info, o as usize))
        } else {
            None
        };
        wide.or_else(|| u32_at(info, ansi_field).and_then(|o| cstr_at(info, o as usize)))
    };
    let suffix = pick(0x18, 0x20).unwrap_or_default();

    if flags & 0x1 != 0 {
        if let Some(base) = pick(0x10, 0x1C) {
            return Some(join_path(&base, &suffix));
        }
    }
    if flags & 0x2 != 0 {
        let cnrl = info.get(u32_at(info, 0x14)? as usize..)?;
        let net_offset = u32_at(cnrl, 8)? as usize;
        let wide = if net_offset > 0x14 {
            u32_at(cnrl, 0x14).and_then(|o| wstr_at(cnrl, o as usize))
        } else {
            None
        };
        let net = wide.or_else(|| cstr_at(cnrl, net_offset))?;
        return Some(join_path(&net, &suffix));
    }
    None
}

fn read_lnk_string(d: &[u8], off: &mut usize, unicode: bool) -> Option<String> {
    let count = u16_at(d, *off)? as usize;
    *off += 2;
    let len = if unicode { count * 2 } else { count };
    let bytes = d.get(*off..*off + len)?;
    *off += len;
    Some(if unicode {
        wstr_from(bytes)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    })
}

struct Shortcut {
    targets: Vec<String>,
    args: String,
}

fn parse_lnk(d: &[u8], lnk_dir: &Path) -> Option<Shortcut> {
    if d.len() < 0x4C || d[0..4] != [0x4C, 0, 0, 0] {
        return None;
    }
    let flags = u32_at(d, 0x14)?;
    let unicode = flags & 0x80 != 0;
    let mut off = 0x4C;
    let mut targets = Vec::new();

    if flags & 0x01 != 0 {
        off += 2 + u16_at(d, off)? as usize;
    }
    if flags & 0x02 != 0 {
        let size = u32_at(d, off)? as usize;
        if let Some(path) = d.get(off..off + size).and_then(link_info_path) {
            targets.push(path);
        }
        off += size;
    }

    let mut relative = None;
    let mut args = String::new();
    for bit in [0x04, 0x08, 0x10, 0x20, 0x40] {
        if flags & bit == 0 {
            continue;
        }
        let Some(s) = read_lnk_string(d, &mut off, unicode) else { break };
        match bit {
            0x08 => relative = Some(s),
            0x20 => args = s,
            _ => {}
        }
    }

    while let Some(size) = u32_at(d, off).map(|s| s as usize) {
        if size < 8 {
            break;
        }
        if u32_at(d, off + 4) == Some(0xA000_0001) && size >= 0x314 {
            let wide = d.get(off + 268..off + 788).map(wstr_from).unwrap_or_default();
            let target = if wide.is_empty() { cstr_at(d, off + 8).unwrap_or_default() } else { wide };
            if !target.is_empty() {
                targets.push(expand_env(&target));
            }
        }
        off += size;
    }

    if let Some(rel) = relative.filter(|r| !r.is_empty()) {
        let rel = expand_env(&rel);
        let path = Path::new(&rel);
        let joined = if path.is_absolute() { path.to_path_buf() } else { lnk_dir.join(path) };
        targets.push(joined.to_string_lossy().into_owned());
    }

    Some(Shortcut { targets, args })
}

fn read_dir_entries(loc: &Location, legacy: bool, entries: &mut Vec<StartupEntry>) {
    let dir = if legacy { loc.legacy_dir() } else { loc.dir.clone() };
    let Ok(read_dir) = fs::read_dir(&dir) else { return };
    let hive = loc.hive();
    let source_id = if legacy { format!("{}{LEGACY_SUFFIX}", loc.id) } else { loc.id.to_string() };

    for item in read_dir.filter_map(Result::ok) {
        let path = item.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else { continue };
        if file_name.eq_ignore_ascii_case("desktop.ini") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&file_name)
            .to_string();
        let is_lnk = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("lnk"));

        let (exe_path, args, file_exists) = if is_lnk {
            let shortcut = fs::read(&path).ok().and_then(|d| parse_lnk(&d, &dir));
            match shortcut {
                Some(s) if !s.targets.is_empty() => {
                    let existing = s.targets.iter().find(|t| Path::new(t).exists()).cloned();
                    let exists = existing.is_some();
                    let target = existing.unwrap_or_else(|| s.targets[0].clone());
                    (target, s.args, exists)
                }
                Some(s) => (path.to_string_lossy().into_owned(), s.args, true),
                None => (path.to_string_lossy().into_owned(), String::new(), true),
            }
        } else {
            (path.to_string_lossy().into_owned(), String::new(), true)
        };

        let enabled = !legacy && !approved::is_disabled(&hive, approved::STARTUP_FOLDER, &file_name);
        let command = if args.is_empty() { exe_path.clone() } else { format!("\"{exe_path}\" {args}") };

        entries.push(StartupEntry {
            id: make_id(&source_id, &file_name),
            name,
            key: file_name,
            command,
            exe_path,
            args,
            source: Source::Folder,
            source_id: source_id.clone(),
            scope: if loc.machine { Scope::AllUsers } else { Scope::CurrentUser },
            enabled,
            file_exists,
            entry_type: EntryType::Run,
        });
    }
}

pub fn read_folder_entries() -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    for loc in locations() {
        read_dir_entries(&loc, false, &mut entries);
        read_dir_entries(&loc, true, &mut entries);
    }
    entries
}

fn io_err(e: std::io::Error) -> String {
    super::resolve::reg_err(e)
}

pub fn set_folder_entry_enabled(source_id: &str, file_name: &str, enabled: bool) -> Result<(), String> {
    let (loc, legacy) = find(source_id)?;
    let hive = loc.hive();
    if legacy {
        if !enabled {
            return Ok(());
        }
        fs::rename(loc.legacy_dir().join(file_name), loc.dir.join(file_name)).map_err(io_err)?;
        let _ = fs::remove_dir(loc.legacy_dir());
    }
    approved::set(&hive, approved::STARTUP_FOLDER, file_name, enabled)
}

pub fn delete_folder_entry(source_id: &str, file_name: &str) -> Result<(), String> {
    let (loc, legacy) = find(source_id)?;
    let dir = if legacy { loc.legacy_dir() } else { loc.dir.clone() };
    fs::remove_file(dir.join(file_name)).map_err(io_err)?;
    if legacy {
        let _ = fs::remove_dir(loc.legacy_dir());
    }
    approved::remove(&loc.hive(), approved::STARTUP_FOLDER, file_name);
    Ok(())
}
