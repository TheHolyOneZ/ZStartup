use super::resolve::{make_id, reg_err};
use crate::models::{EntryType, Scope, Source, StartupEntry};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

const APPMODEL: &str =
    r"Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel";
const SOURCE_ID: &str = "appx";

const STATE_DISABLED_BY_USER: u32 = 1;
const STATE_ENABLED: u32 = 2;
const STATE_DISABLED_BY_POLICY: u32 = 3;
const STATE_ENABLED_BY_POLICY: u32 = 4;

struct Task {
    task_id: String,
    executable: String,
    display_name: String,
    enabled_by_default: bool,
}

fn unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let mut from = 0;
    while let Some(pos) = tag[from..].find(&needle) {
        let start = from + pos;
        let preceded_by_space = tag[..start].chars().last().is_some_and(char::is_whitespace);
        let value_start = start + needle.len();
        if preceded_by_space {
            let end = tag[value_start..].find('"')?;
            return Some(unescape(&tag[value_start..value_start + end]));
        }
        from = value_start;
    }
    None
}

fn tag_around(xml: &str, pos: usize) -> Option<&str> {
    let start = xml[..pos].rfind('<')?;
    let end = pos + xml[pos..].find('>')?;
    Some(&xml[start..=end])
}

fn parse_tasks(xml: &str) -> Vec<Task> {
    let mut tasks = Vec::new();
    let mut from = 0;
    while let Some(rel) = xml[from..].find("\"windows.startupTask\"") {
        let pos = from + rel;
        from = pos + 1;
        let Some(ext_tag) = tag_around(xml, pos) else { continue };
        let executable = attr(ext_tag, "Executable").or_else(|| {
            let app = xml[..pos].rfind("<Application ")?;
            attr(tag_around(xml, app + 1)?, "Executable")
        });
        let Some(task_rel) = xml[pos..].find("StartupTask ") else { continue };
        let Some(task_tag) = tag_around(xml, pos + task_rel) else { continue };
        let Some(task_id) = attr(task_tag, "TaskId") else { continue };
        tasks.push(Task {
            task_id,
            executable: executable.unwrap_or_default(),
            display_name: attr(task_tag, "DisplayName").unwrap_or_default(),
            enabled_by_default: attr(task_tag, "Enabled").is_some_and(|v| v.eq_ignore_ascii_case("true")),
        });
    }
    tasks
}

fn element_text(xml: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let start = xml.find(&open)? + open.len();
    let end = start + xml[start..].find('<')?;
    Some(unescape(xml[start..end].trim()))
}

fn load_indirect(source: &str) -> Option<String> {
    use windows_sys::Win32::UI::Shell::SHLoadIndirectString;
    let wide: Vec<u16> = source.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = [0u16; 512];
    let hr = unsafe {
        SHLoadIndirectString(wide.as_ptr(), buf.as_mut_ptr(), buf.len() as u32, std::ptr::null_mut())
    };
    if hr < 0 {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let s = String::from_utf16_lossy(&buf[..len]).trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn resolve_name(raw: &str, full_name: &str, package_name: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let Some(res) = raw.strip_prefix("ms-resource:") else {
        return Some(raw.to_string());
    };
    let uris = if res.starts_with("//") {
        vec![format!("ms-resource:{res}")]
    } else if res.starts_with('/') {
        vec![format!("ms-resource://{package_name}{res}")]
    } else {
        vec![
            format!("ms-resource://{package_name}/Resources/{res}"),
            format!("ms-resource://{package_name}/{res}"),
        ]
    };
    uris.iter()
        .find_map(|u| load_indirect(&format!("@{{{full_name}?{u}}}")))
}

fn task_state(hkcu: &RegKey, family: &str, task_id: &str) -> Option<u32> {
    hkcu.open_subkey(format!(r"{APPMODEL}\SystemAppData\{family}\{task_id}"))
        .and_then(|k| k.get_value::<u32, _>("State"))
        .ok()
}

fn reachable(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(_) => true,
        Err(e) => e.kind() == std::io::ErrorKind::PermissionDenied,
    }
}

pub fn read_packaged_entries() -> Vec<StartupEntry> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(packages) = hkcu.open_subkey(format!(r"{APPMODEL}\Repository\Packages")) else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    let mut entries = Vec::new();

    for full_name in packages.enum_keys().filter_map(Result::ok) {
        let parts: Vec<&str> = full_name.split('_').collect();
        if parts.len() < 5 {
            continue;
        }
        let package_name = parts[0];
        let family = format!("{package_name}_{}", parts[parts.len() - 1]);

        let Ok(pkg) = packages.open_subkey(&full_name) else { continue };
        let Ok(root) = pkg.get_value::<String, _>("PackageRootFolder") else { continue };
        let root = Path::new(&root);
        let Ok(xml) = fs::read_to_string(root.join("AppxManifest.xml")) else { continue };
        if !xml.contains("windows.startupTask") {
            continue;
        }

        let app_name = element_text(&xml, "DisplayName")
            .and_then(|n| resolve_name(&n, &full_name, package_name));

        for task in parse_tasks(&xml) {
            let key = format!(r"{family}\{}", task.task_id);
            if !seen.insert(key.to_lowercase()) {
                continue;
            }
            let enabled = match task_state(&hkcu, &family, &task.task_id) {
                Some(state) => state == STATE_ENABLED || state == STATE_ENABLED_BY_POLICY,
                None => task.enabled_by_default,
            };
            let name = resolve_name(&task.display_name, &full_name, package_name)
                .or_else(|| app_name.clone())
                .unwrap_or_else(|| package_name.to_string());
            let exe = if task.executable.is_empty() {
                root.to_path_buf()
            } else {
                root.join(task.executable.replace('/', "\\"))
            };
            let exe_path = exe.to_string_lossy().into_owned();

            entries.push(StartupEntry {
                id: make_id(SOURCE_ID, &key),
                name,
                key,
                command: exe_path.clone(),
                file_exists: reachable(&exe),
                exe_path,
                args: String::new(),
                source: Source::Packaged,
                source_id: SOURCE_ID.into(),
                scope: Scope::CurrentUser,
                enabled,
                entry_type: EntryType::Run,
            });
        }
    }
    entries
}

pub fn set_packaged_entry_enabled(key: &str, enabled: bool) -> Result<(), String> {
    let (family, task_id) = key.split_once('\\').ok_or("Invalid app startup task")?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Some(state) = task_state(&hkcu, family, task_id) {
        if state == STATE_DISABLED_BY_POLICY || state == STATE_ENABLED_BY_POLICY {
            return Err("This app's startup is controlled by a group policy and can't be changed here.".into());
        }
    }
    let (task, _) = hkcu
        .create_subkey(format!(r"{APPMODEL}\SystemAppData\{family}\{task_id}"))
        .map_err(reg_err)?;
    let state = if enabled { STATE_ENABLED } else { STATE_DISABLED_BY_USER };
    task.set_value("State", &state).map_err(reg_err)
}
