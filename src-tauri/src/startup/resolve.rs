use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub fn make_id(source_id: &str, key: &str) -> String {
    let mut h = Sha256::new();
    h.update(source_id);
    h.update(key);
    hex::encode(h.finalize())[..16].to_string()
}

pub fn reg_err(e: std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        "Access denied. Restart ZStartup as Administrator to change system-wide entries.".into()
    } else {
        e.to_string()
    }
}

pub fn expand_env(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('%') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let var = &after[..end];
        match std::env::var(var).ok().filter(|_| !var.is_empty()) {
            Some(value) => {
                out.push_str(&value);
                rest = &after[end + 1..];
            }
            None => {
                out.push('%');
                out.push_str(var);
                rest = &after[end..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(root) = std::env::var("SystemRoot") {
        let root = PathBuf::from(root);
        dirs.push(root.join("System32"));
        dirs.push(root);
    }
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    dirs
}

pub fn locate(candidate: &str, default_ext: &str) -> Option<PathBuf> {
    let c = candidate.trim().trim_matches('"');
    if c.is_empty() {
        return None;
    }
    let mut variants = vec![c.to_string()];
    if Path::new(c).extension().is_none() {
        variants.push(format!("{c}.{default_ext}"));
    }
    let has_dir = c.contains(['\\', '/', ':']);
    let dirs = if has_dir { Vec::new() } else { search_dirs() };
    for v in variants {
        if has_dir {
            let p = PathBuf::from(&v);
            if p.is_file() {
                return Some(p);
            }
        } else if let Some(p) = dirs.iter().map(|d| d.join(&v)).find(|p| p.is_file()) {
            return Some(p);
        }
    }
    None
}

pub struct Resolved {
    pub exe_path: String,
    pub args: String,
    pub exists: bool,
}

pub fn resolve_command(raw: &str) -> Resolved {
    let expanded = expand_env(raw.trim());
    let (exe, args, located) = split_command(expanded.trim());
    let mut res = Resolved {
        exists: located.is_some(),
        exe_path: located.map(|p| p.to_string_lossy().into_owned()).unwrap_or(exe),
        args,
    };
    if res.exists {
        check_host_target(&mut res);
    }
    res
}

fn split_command(cmd: &str) -> (String, String, Option<PathBuf>) {
    if let Some(stripped) = cmd.strip_prefix('"') {
        let (exe, rest) = match stripped.find('"') {
            Some(end) => (&stripped[..end], stripped[end + 1..].trim()),
            None => (stripped, ""),
        };
        return (exe.to_string(), rest.to_string(), locate(exe, "exe"));
    }
    let parts: Vec<&str> = cmd.split(' ').collect();
    for i in 1..=parts.len() {
        let candidate = parts[..i].join(" ");
        if let Some(found) = locate(&candidate, "exe") {
            return (candidate, parts[i..].join(" ").trim().to_string(), Some(found));
        }
    }
    let cut = parts
        .iter()
        .position(|p| p.to_ascii_lowercase().ends_with(".exe"))
        .map_or(1, |i| i + 1);
    (
        parts[..cut].join(" "),
        parts[cut..].join(" ").trim().to_string(),
        None,
    )
}

fn check_host_target(res: &mut Resolved) {
    let host = Path::new(&res.exe_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if host != "rundll32.exe" {
        return;
    }
    let arg = res.args.trim();
    let target = match arg.strip_prefix('"') {
        Some(s) => s.split('"').next().unwrap_or(""),
        None => arg.split(',').next().unwrap_or(""),
    };
    let target = target.split(',').next().unwrap_or("").trim();
    if target.is_empty() {
        return;
    }
    match locate(target, "dll") {
        Some(p) => res.exe_path = p.to_string_lossy().into_owned(),
        None => {
            res.exe_path = target.to_string();
            res.exists = false;
        }
    }
}
