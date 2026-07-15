use crate::esi;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct MutationFailure {
    pub path: String,
    pub error: String,
}

#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct BatchMutationResult {
    pub succeeded: Vec<String>,
    pub failed: Vec<MutationFailure>,
}

#[cfg(target_os = "windows")]
fn is_eve_client_running() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    let mut found = false;
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_entry {
        let end = entry
            .szExeFile
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(entry.szExeFile.len());
        let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
        if name.eq_ignore_ascii_case("exefile.exe") || name.eq_ignore_ascii_case("eve.exe") {
            found = true;
            break;
        }
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }

    unsafe { CloseHandle(snapshot) };
    found
}

#[cfg(not(target_os = "windows"))]
fn is_eve_client_running() -> bool {
    false
}

fn ensure_eve_closed() -> Result<(), String> {
    if is_eve_client_running() {
        Err("EVE Online is running. Close every EVE client before changing settings so the client cannot overwrite your changes.".into())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn is_eve_running() -> bool {
    is_eve_client_running()
}

fn emit_data_changed(app: &tauri::AppHandle) {
    let _ = app.emit("data-changed", ());
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("eve-wrench");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_path = parent.join(format!(
        ".{}.{}.{}.tmp",
        file_name,
        std::process::id(),
        nonce
    ));
    let result = (|| -> Result<(), String> {
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|e| format!("Failed to create temporary file: {}", e))?;
        temp.write_all(data)
            .map_err(|e| format!("Failed to write temporary file: {}", e))?;
        temp.sync_all()
            .map_err(|e| format!("Failed to flush temporary file: {}", e))?;
        drop(temp);
        replace_file(&temp_path, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

#[cfg(not(target_os = "windows"))]
fn replace_file(temp_path: &Path, destination: &Path) -> Result<(), String> {
    fs::rename(temp_path, destination).map_err(|e| {
        format!(
            "Failed to replace {} atomically: {}",
            destination.display(),
            e
        )
    })?;
    if let Some(parent) = destination.parent() {
        if let Ok(directory) = fs::File::open(parent) {
            let _ = directory.sync_all();
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn replace_file(temp_path: &Path, destination: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;
    if !destination.exists() {
        return fs::rename(temp_path, destination)
            .map_err(|e| format!("Failed to install {}: {}", destination.display(), e));
    }
    let destination_wide: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let temp_wide: Vec<u16> = temp_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let replaced = unsafe {
        ReplaceFileW(
            destination_wide.as_ptr(),
            temp_wide.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        return Err(format!(
            "Failed to replace {} atomically: {}",
            destination.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Server {
    Tranquility,
    Singularity,
    Thunderdome,
    Serenity,
}

impl Server {
    fn from_folder_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        if lower.contains("tranquility") {
            Some(Server::Tranquility)
        } else if lower.contains("singularity") {
            Some(Server::Singularity)
        } else if lower.contains("thunderdome") {
            Some(Server::Thunderdome)
        } else if lower.contains("serenity") {
            Some(Server::Serenity)
        } else {
            None
        }
    }

    fn supports_esi(&self) -> bool {
        matches!(self, Server::Tranquility)
    }

    fn display_name(&self) -> &'static str {
        match self {
            Server::Tranquility => "Tranquility",
            Server::Singularity => "Singularity",
            Server::Thunderdome => "Thunderdome",
            Server::Serenity => "Serenity",
        }
    }

    fn short_name(&self) -> &'static str {
        match self {
            Server::Tranquility => "TQ",
            Server::Singularity => "SISI",
            Server::Thunderdome => "TD",
            Server::Serenity => "CN",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            Server::Tranquility => "#00d4aa",
            Server::Singularity => "#f0b429",
            Server::Thunderdome => "#f85149",
            Server::Serenity => "#a78bfa",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SettingsKind {
    User,
    Char,
}

#[derive(Serialize, Debug, Clone)]
pub struct ServerInfo {
    pub id: Server,
    pub name: String,
    pub short_name: String,
    pub color: String,
    pub supports_esi: bool,
    pub brackets_always_show: bool,
    pub server_path: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CharacterDetails {
    pub name: String,
    pub corporation: Option<String>,
    pub portrait_url: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct SettingsEntry {
    pub path: String,
    pub id: String,
    pub kind: SettingsKind,
    pub server: Server,
    pub profile: String,
    pub display_name: String,
    pub character: Option<CharacterDetails>,
    pub alias: Option<String>,
    pub modified_time: u64,
    pub relative_time: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ProfileData {
    pub name: String,
    pub path: String,
    pub accounts: Vec<SettingsEntry>,
    pub characters: Vec<SettingsEntry>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ServerData {
    pub info: ServerInfo,
    pub profiles: Vec<ProfileData>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BackupEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub timestamp: u64,
    pub kind: SettingsKind,
    pub original_id: String,
    pub original_name: Option<String>,
    pub server: Server,
    pub profile: String,
    pub display_name: String,
    pub relative_time: String,
}

fn backup_entry_id(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn settings_path_context(path: &Path) -> Result<(Server, String), String> {
    let profile_dir = path
        .parent()
        .ok_or("Could not determine settings profile")?;
    let profile_name = profile_dir
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("settings_"))
        .filter(|name| !name.is_empty())
        .ok_or("Settings file is not inside a settings profile")?;
    let server_name = profile_dir
        .parent()
        .and_then(|directory| directory.file_name())
        .and_then(|name| name.to_str())
        .ok_or("Could not determine settings server")?;
    let server = Server::from_folder_name(server_name)
        .ok_or_else(|| format!("Unrecognized EVE server folder: {}", server_name))?;
    Ok((server, profile_name.to_string()))
}

#[derive(Serialize, Debug, Clone)]
pub struct AppData {
    pub servers: Vec<ServerData>,
    pub backups: Vec<BackupEntry>,
}

fn eve_settings_root(custom_path: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = custom_path {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Some(path);
        }
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|h| h.join("Library/Application Support/CCP/EVE"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir().map(|d| d.join("CCP/EVE"))
    }
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|h| {
            h.join(".local/share/Steam/steamapps/compatdata/8500/pfx/drive_c/users/steamuser/AppData/Local/CCP/EVE")
        })
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

fn backup_directory_for_path(source_path: &Path) -> Result<PathBuf, String> {
    let profile_dir = source_path
        .parent() // settings_profile dir (e.g., settings_Default)
        .ok_or("Could not determine profile directory")?;

    let path = profile_dir.join("backups");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

// Splits a "core_{kind}_{id}.dat" filename into (kind, id).
fn parse_core_filename(path: &Path) -> Result<(&str, &str), String> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?;
    let stem = filename
        .strip_prefix("core_")
        .and_then(|s| s.strip_suffix(".dat"))
        .ok_or("Invalid settings file")?;
    stem.split_once('_')
        .ok_or_else(|| "Invalid settings file format".to_string())
}

// Copies a settings file into its profile's backups folder using the shared
// "{name}_{kind}_{id}_{timestamp}.bak" naming scheme, so every backup shows
// up in the app's backup list. Returns the backup path and its timestamp.
fn auto_backup(path: &Path, name: &str) -> Result<(PathBuf, u64), String> {
    let (kind_str, id) = parse_core_filename(path)?;
    let safe_name: String = name
        .chars()
        .take(80)
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | '(' | ')') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches([' ', '.'])
        .to_string();
    let safe_name = if safe_name.is_empty() {
        "backup".to_string()
    } else {
        safe_name
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let backup_dir = backup_directory_for_path(path)?;
    let dest = (0..1000)
        .map(|suffix| {
            let label = if suffix == 0 {
                safe_name.clone()
            } else {
                format!("{} ({})", safe_name, suffix + 1)
            };
            backup_dir.join(format!("{}_{}_{}_{}.bak", label, kind_str, id, timestamp))
        })
        .find(|candidate| !candidate.exists())
        .ok_or("Could not allocate a unique backup name")?;
    fs::copy(path, &dest).map_err(|e| e.to_string())?;
    if safe_name.starts_with("pre-") {
        prune_automatic_backups(&backup_dir, kind_str, id, 25);
    }
    Ok((dest, timestamp))
}

fn prune_automatic_backups(directory: &Path, kind: &str, id: &str, keep: usize) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut candidates: Vec<(u64, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let filename = path.file_name()?.to_str()?;
            let stem = filename.strip_suffix(".bak")?;
            let parts: Vec<&str> = stem.rsplitn(4, '_').collect();
            if parts.len() != 4
                || parts[2] != kind
                || parts[1] != id
                || !parts[3].starts_with("pre-")
            {
                return None;
            }
            Some((parts[0].parse().unwrap_or(0), path))
        })
        .collect();
    candidates.sort_by(|a, b| b.cmp(a));
    for (_, path) in candidates.into_iter().skip(keep) {
        let _ = fs::remove_file(path);
    }
}

fn aliases_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut path = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("aliases.json");
    Ok(path)
}

fn load_aliases(app: &tauri::AppHandle) -> HashMap<String, String> {
    let path = match aliases_file(app) {
        Ok(p) => p,
        Err(_) => return HashMap::new(),
    };

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn save_aliases(app: &tauri::AppHandle, aliases: &HashMap<String, String>) -> Result<(), String> {
    let path = aliases_file(app)?;
    let content = serde_json::to_string_pretty(aliases).map_err(|e| e.to_string())?;
    atomic_write(&path, content.as_bytes())?;
    Ok(())
}

fn read_brackets_setting(server_path: &PathBuf) -> bool {
    // Check all profile folders (settings_*) for the setting
    if let Ok(entries) = fs::read_dir(server_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() && name.starts_with("settings_") {
                let prefs_path = path.join("prefs.ini");
                if let Ok(content) = fs::read_to_string(&prefs_path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("bracketsAlwaysShowShipText=") {
                            if let Some(value) = trimmed.strip_prefix("bracketsAlwaysShowShipText=")
                            {
                                if value.trim() == "1" {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn write_brackets_setting(server_path: &PathBuf, enabled: bool) -> Result<(), String> {
    let setting_line = format!(
        "bracketsAlwaysShowShipText={}",
        if enabled { "1" } else { "0" }
    );

    // Apply to all profile folders (settings_*)
    let entries = fs::read_dir(server_path).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() && name.starts_with("settings_") {
            let prefs_path = path.join("prefs.ini");

            let content = if prefs_path.exists() {
                let existing = fs::read_to_string(&prefs_path).map_err(|e| e.to_string())?;
                let mut found = false;
                let mut lines: Vec<String> = existing
                    .lines()
                    .map(|line| {
                        if line.trim().starts_with("bracketsAlwaysShowShipText=") {
                            found = true;
                            setting_line.clone()
                        } else {
                            line.to_string()
                        }
                    })
                    .collect();

                if !found {
                    lines.push(setting_line.clone());
                }
                lines.join("\n")
            } else {
                setting_line.clone()
            };

            atomic_write(&prefs_path, content.as_bytes())?;
        }
    }
    Ok(())
}

fn format_relative_time(timestamp: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

struct RawSettingsFile {
    path: String,
    id: String,
    kind: SettingsKind,
    server: Server,
    profile: String,
    modified_time: u64,
}

fn parse_settings_file(
    path: &PathBuf,
    server: Server,
    profile_name: &str,
) -> Option<RawSettingsFile> {
    let filename = path.file_name()?.to_str()?;

    if !filename.starts_with("core_") || !filename.ends_with(".dat") {
        return None;
    }

    let stem = filename
        .trim_start_matches("core_")
        .trim_end_matches(".dat");
    let (kind_str, id) = stem.split_once('_')?;

    if id.is_empty() || id.parse::<u64>().is_err() {
        return None;
    }

    let kind = match kind_str {
        "user" => SettingsKind::User,
        "char" => SettingsKind::Char,
        _ => return None,
    };

    let modified_time = fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Some(RawSettingsFile {
        path: path.to_string_lossy().into_owned(),
        id: id.to_string(),
        kind,
        server,
        profile: profile_name.to_string(),
        modified_time,
    })
}

type ScanResult = (HashMap<Server, Vec<ProfileData>>, HashMap<Server, PathBuf>);

fn scan_installations(custom_eve_path: Option<&str>) -> Result<ScanResult, String> {
    let root = eve_settings_root(custom_eve_path).ok_or("EVE settings directory not found")?;

    if !root.exists() {
        return Ok((HashMap::new(), HashMap::new()));
    }

    let mut server_profiles: HashMap<Server, Vec<ProfileData>> = HashMap::new();
    let mut server_paths: HashMap<Server, PathBuf> = HashMap::new();
    let entries = fs::read_dir(&root).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if !path.is_dir() {
            continue;
        }

        let server = match Server::from_folder_name(name) {
            Some(s) => s,
            None => continue,
        };

        server_paths.insert(server, path.clone());

        let sub_entries = match fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for sub_entry in sub_entries.flatten() {
            let sub_path = sub_entry.path();
            let sub_name = sub_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            if !sub_path.is_dir() || !sub_name.starts_with("settings_") {
                continue;
            }

            let profile_name = sub_name.trim_start_matches("settings_");
            let mut accounts: Vec<RawSettingsFile> = Vec::new();
            let mut characters: Vec<RawSettingsFile> = Vec::new();

            if let Ok(files) = fs::read_dir(&sub_path) {
                for file in files.flatten() {
                    if let Some(settings) = parse_settings_file(&file.path(), server, profile_name)
                    {
                        match settings.kind {
                            SettingsKind::User => accounts.push(settings),
                            SettingsKind::Char => characters.push(settings),
                        }
                    }
                }
            }

            accounts.sort_by(|a, b| a.id.cmp(&b.id));
            characters.sort_by(|a, b| a.id.cmp(&b.id));

            let profile = ProfileData {
                name: profile_name.to_string(),
                path: sub_path.to_string_lossy().into_owned(),
                accounts: accounts
                    .into_iter()
                    .map(|f| SettingsEntry {
                        display_name: f.id.clone(),
                        relative_time: format_relative_time(f.modified_time),
                        modified_time: f.modified_time,
                        path: f.path,
                        id: f.id,
                        kind: f.kind,
                        server: f.server,
                        profile: f.profile,
                        character: None,
                        alias: None,
                    })
                    .collect(),
                characters: characters
                    .into_iter()
                    .map(|f| SettingsEntry {
                        display_name: f.id.clone(),
                        relative_time: format_relative_time(f.modified_time),
                        modified_time: f.modified_time,
                        path: f.path,
                        id: f.id,
                        kind: f.kind,
                        server: f.server,
                        profile: f.profile,
                        character: None,
                        alias: None,
                    })
                    .collect(),
            };

            server_profiles.entry(server).or_default().push(profile);
        }
    }

    for profiles in server_profiles.values_mut() {
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
    }

    Ok((server_profiles, server_paths))
}

fn scan_backups(custom_eve_path: Option<&str>) -> Result<Vec<BackupEntry>, String> {
    let eve_root = match eve_settings_root(custom_eve_path) {
        Some(r) => r,
        None => return Ok(Vec::new()),
    };

    let mut backups = Vec::new();

    let server_dirs = match fs::read_dir(&eve_root) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };

    for server_entry in server_dirs.flatten() {
        let server_path = server_entry.path();
        if !server_path.is_dir() {
            continue;
        }
        let Some(server) = server_path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(Server::from_folder_name)
        else {
            continue;
        };

        let profile_dirs = match fs::read_dir(&server_path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for profile_entry in profile_dirs.flatten() {
            let profile_path = profile_entry.path();
            if !profile_path.is_dir() {
                continue;
            }

            let dir_name = profile_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !dir_name.starts_with("settings_") {
                continue;
            }
            let profile = dir_name.trim_start_matches("settings_").to_string();

            let backup_dir = profile_path.join("backups");
            let entries = match fs::read_dir(&backup_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let filename = match path.file_name().and_then(|n| n.to_str()) {
                    Some(f) if f.ends_with(".bak") => f,
                    _ => continue,
                };

                let stem = filename.trim_end_matches(".bak");
                let parts: Vec<&str> = stem.rsplitn(4, '_').collect();

                if parts.len() < 4 {
                    continue;
                }

                let timestamp = parts[0].parse::<u64>().unwrap_or(0);
                let original_id = parts[1].to_string();
                let kind = match parts[2] {
                    "user" => SettingsKind::User,
                    "char" => SettingsKind::Char,
                    _ => continue,
                };
                let name = parts[3].to_string();
                let path_string = path.to_string_lossy().into_owned();

                backups.push(BackupEntry {
                    id: backup_entry_id(&path),
                    display_name: name.clone(),
                    name,
                    path: path_string,
                    timestamp,
                    kind,
                    original_id,
                    original_name: None,
                    server,
                    profile: profile.clone(),
                    relative_time: format_relative_time(timestamp),
                });
            }
        }
    }

    backups.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
    Ok(backups)
}

#[tauri::command]
pub async fn get_app_data(
    app: tauri::AppHandle,
    custom_eve_path: Option<String>,
) -> Result<AppData, String> {
    let (mut server_profiles, server_paths) = scan_installations(custom_eve_path.as_deref())?;
    let mut backups = scan_backups(custom_eve_path.as_deref())?;
    let aliases = load_aliases(&app);

    for profiles in server_profiles.values_mut() {
        for profile in profiles.iter_mut() {
            for account in profile.accounts.iter_mut() {
                if let Some(alias) = aliases.get(&account.id) {
                    account.alias = Some(alias.clone());
                    account.display_name = alias.clone();
                }
            }
            for character in profile.characters.iter_mut() {
                if let Some(alias) = aliases.get(&character.id) {
                    character.alias = Some(alias.clone());
                    character.display_name = alias.clone();
                }
            }
        }
    }

    for server in [Server::Tranquility, Server::Singularity] {
        if let Some(profiles) = server_profiles.get_mut(&server) {
            for profile in profiles.iter_mut() {
                for character in profile.characters.iter_mut() {
                    if let Ok(char_id) = character.id.parse::<i64>() {
                        if char_id >= 90_000_000 {
                            if let Ok(info) = esi::get_character(char_id).await {
                                character.display_name = info.name.clone();
                                character.character = Some(CharacterDetails {
                                    name: info.name,
                                    corporation: info.corporation_name,
                                    portrait_url: format!(
                                        "https://images.evetech.net/characters/{}/portrait?size=64",
                                        char_id
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let mut servers: Vec<ServerData> = Vec::new();
    let all_servers = [
        Server::Tranquility,
        Server::Singularity,
        Server::Thunderdome,
        Server::Serenity,
    ];

    for server in all_servers {
        if let Some(profiles) = server_profiles.remove(&server) {
            if !profiles.is_empty() {
                let server_path = server_paths.get(&server).cloned().unwrap_or_default();
                let brackets_always_show = read_brackets_setting(&server_path);
                servers.push(ServerData {
                    info: ServerInfo {
                        id: server,
                        name: server.display_name().to_string(),
                        short_name: server.short_name().to_string(),
                        color: server.color().to_string(),
                        supports_esi: server.supports_esi(),
                        brackets_always_show,
                        server_path: server_path.to_string_lossy().into_owned(),
                    },
                    profiles,
                });
            }
        }
    }

    // Enrich backups with entity names from settings entries
    for backup in backups.iter_mut() {
        for server in &servers {
            for profile in &server.profiles {
                let entries = if backup.kind == SettingsKind::User {
                    &profile.accounts
                } else {
                    &profile.characters
                };
                if let Some(entry) = entries.iter().find(|e| e.id == backup.original_id) {
                    backup.original_name = Some(entry.display_name.clone());
                    break;
                }
            }
            if backup.original_name.is_some() {
                break;
            }
        }
    }

    Ok(AppData { servers, backups })
}

// ── Probe formations ─────────────────────────────────────────────────────
//
// Stored in the account (core_user) file under ui -> probescanning.customFormations
// as (timestamp, {id: (name, [((x, y, z), range), ...])}). Every leaf setting is
// wrapped in a (FILETIME timestamp, value) tuple; positions and ranges are meters.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FormationProbe {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub range: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProbeFormation {
    pub id: i64,
    pub name: String,
    pub probes: Vec<FormationProbe>,
}

#[derive(Serialize, Debug, Clone)]
pub struct FormationSnapshot {
    pub formations: Vec<ProbeFormation>,
    pub file_sha256: String,
}
#[derive(Serialize, Debug, Clone)]
pub struct FormationWriteResult {
    pub file_sha256: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortableFormation {
    pub schema_version: u32,
    pub name: String,
    pub probes: Vec<PortableProbe>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortableProbe {
    pub x_km: f64,
    pub y_km: f64,
    pub z_km: f64,
    pub range_au: f64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortableFormationPack {
    pub schema_version: u32,
    pub formations: Vec<PortableFormation>,
}

const UI_KEY: &str = "bytes:ui";
const FORMATIONS_KEY: &str = "bytes:probescanning.customFormations";
const SELECTED_FORMATION_KEY: &str = "bytes:probescanning.selectedFormationID";
const AU_METERS: f64 = 149_597_870_700.0;
const MAX_CUSTOM_FORMATIONS: usize = 10;
const MAX_PROBES_PER_FORMATION: usize = 8;
const MAX_FORMATION_NAME_CHARS: usize = 64;
// Union of Core Scanner Probe (0.25–32 AU) and Combat Scanner Probe
// (0.5–64 AU) ranges. Probe type is not stored in a custom formation.
const VALID_SCAN_RANGES_AU: [f64; 9] = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];

// Windows FILETIME: 100ns intervals since 1601-01-01, which is what EVE
// stamps on every settings value.
fn filetime_now() -> u64 {
    let unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (unix_secs + 11_644_473_600) * 10_000_000
}

fn load_settings_json(path: &Path) -> Result<(serde_json::Value, bool), String> {
    let bytes = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    decode_settings_json(&bytes)
}

fn decode_settings_json(bytes: &[u8]) -> Result<(serde_json::Value, bool), String> {
    let decoded = blue_marshal::decode(bytes)
        .map_err(|e| format!("Failed to decode settings file: {}", e))?;
    Ok((blue_marshal::to_json(&decoded.value), decoded.had_crc))
}

fn validate_formations(formations: &[ProbeFormation]) -> Result<(), String> {
    if formations.len() > MAX_CUSTOM_FORMATIONS {
        return Err(format!(
            "EVE supports at most {} custom probe formations",
            MAX_CUSTOM_FORMATIONS
        ));
    }
    let mut ids = HashSet::new();
    for (formation_index, formation) in formations.iter().enumerate() {
        if formation.id < 0 || !ids.insert(formation.id) {
            return Err(format!(
                "Formation {} has an invalid or duplicate id",
                formation_index + 1
            ));
        }
        let name = formation.name.trim();
        if name.is_empty() || name.chars().count() > MAX_FORMATION_NAME_CHARS {
            return Err(format!(
                "Formation {} needs a name between 1 and {} characters",
                formation_index + 1,
                MAX_FORMATION_NAME_CHARS
            ));
        }
        if formation.probes.len() > MAX_PROBES_PER_FORMATION {
            return Err(format!(
                "Formation '{}' has more than {} probes",
                name, MAX_PROBES_PER_FORMATION
            ));
        }
        for (probe_index, probe) in formation.probes.iter().enumerate() {
            if !probe.x.is_finite()
                || !probe.y.is_finite()
                || !probe.z.is_finite()
                || !probe.range.is_finite()
            {
                return Err(format!(
                    "Probe {} in '{}' contains a non-finite value",
                    probe_index + 1,
                    name
                ));
            }
            let range_au = probe.range / AU_METERS;
            if !VALID_SCAN_RANGES_AU
                .iter()
                .any(|valid| (range_au - valid).abs() <= 1e-9)
            {
                return Err(format!(
                    "Probe {} in '{}' has an unsupported scan range",
                    probe_index + 1,
                    name
                ));
            }
        }
    }
    Ok(())
}

// Strips blue-marshal's lossless type prefix ("utf8:lol" -> "lol").
fn strip_type_prefix(s: &str) -> &str {
    s.split_once(':').map(|(_, rest)| rest).unwrap_or(s)
}

fn parse_probe(value: &serde_json::Value) -> Option<FormationProbe> {
    let outer = value.get("tuple")?.as_array()?;
    let pos = outer.first()?.get("tuple")?.as_array()?;
    Some(FormationProbe {
        x: pos.first()?.as_f64()?,
        y: pos.get(1)?.as_f64()?,
        z: pos.get(2)?.as_f64()?,
        range: outer.get(1)?.as_f64()?,
    })
}

fn formations_from_settings(root: &serde_json::Value) -> Vec<ProbeFormation> {
    let mut formations = Vec::new();

    let entries = root
        .get(UI_KEY)
        .and_then(|ui| ui.get(FORMATIONS_KEY))
        .and_then(|v| v.get("tuple"))
        .and_then(|t| t.get(1))
        .and_then(|v| v.as_object());

    let Some(entries) = entries else {
        return formations;
    };

    for (key, value) in entries {
        let id = match strip_type_prefix(key).parse::<i64>() {
            Ok(id) => id,
            Err(_) => continue,
        };
        // Negative IDs are client-internal scratch state (e.g. -4 "tempFormation"
        // holding the currently launched probe positions) — not user formations
        if id < 0 {
            continue;
        }
        let Some(tuple) = value.get("tuple").and_then(|t| t.as_array()) else {
            continue;
        };
        let Some(name) = tuple.first().and_then(|v| v.as_str()) else {
            continue;
        };
        let probes = tuple
            .get(1)
            .and_then(|v| v.as_array())
            .map(|list| list.iter().filter_map(parse_probe).collect())
            .unwrap_or_default();

        formations.push(ProbeFormation {
            id,
            name: strip_type_prefix(name).to_string(),
            probes,
        });
    }

    formations.sort_by_key(|f| f.id);
    formations
}

fn formations_into_settings(
    root: &mut serde_json::Value,
    formations: &[ProbeFormation],
) -> Result<(), String> {
    use serde_json::json;

    let mut entries = serde_json::Map::new();

    // Carry over client-internal entries (negative IDs) untouched
    if let Some(existing) = root
        .get(UI_KEY)
        .and_then(|ui| ui.get(FORMATIONS_KEY))
        .and_then(|v| v.get("tuple"))
        .and_then(|t| t.get(1))
        .and_then(|v| v.as_object())
    {
        for (key, value) in existing {
            let id = strip_type_prefix(key).parse::<i64>().unwrap_or(0);
            if id < 0 {
                entries.insert(key.clone(), value.clone());
            }
        }
    }

    for formation in formations {
        let probes: Vec<serde_json::Value> = formation
            .probes
            .iter()
            .map(|p| json!({"tuple": [{"tuple": [p.x, p.y, p.z]}, p.range]}))
            .collect();
        entries.insert(
            format!("int:{}", formation.id),
            json!({"tuple": [format!("utf8:{}", formation.name), probes]}),
        );
    }
    let timestamp = format!("long:{}", filetime_now());

    let root_map = root
        .as_object_mut()
        .ok_or("Settings file has an unexpected structure")?;
    let ui = root_map
        .entry(UI_KEY.to_string())
        .or_insert_with(|| json!({}));
    let ui_map = ui
        .as_object_mut()
        .ok_or("Settings 'ui' section has an unexpected structure")?;

    ui_map.insert(
        FORMATIONS_KEY.to_string(),
        json!({"tuple": [timestamp, entries]}),
    );

    // Keep the selected-formation pointer valid after edits
    if let Some(selected) = ui_map
        .get_mut(SELECTED_FORMATION_KEY)
        .and_then(|v| v.get_mut("tuple"))
        .and_then(|t| t.as_array_mut())
    {
        let current = selected.get(1).and_then(|v| v.as_i64());
        let still_valid = current.is_some_and(|id| formations.iter().any(|f| f.id == id));
        if !still_valid {
            if let Some(slot) = selected.get_mut(1) {
                *slot = json!(formations.first().map(|f| f.id).unwrap_or(0));
            }
        }
    }

    Ok(())
}

fn read_formations_from_file(path: &Path) -> Result<FormationSnapshot, String> {
    let bytes = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let (json, _) = decode_settings_json(&bytes)?;
    Ok(FormationSnapshot {
        formations: formations_from_settings(&json),
        file_sha256: sha256_hex(&bytes),
    })
}

fn write_formations_to_file(
    path: &Path,
    formations: &[ProbeFormation],
    expected_sha256: Option<&str>,
) -> Result<String, String> {
    validate_formations(formations)?;
    let current_bytes = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let current_sha256 = sha256_hex(&current_bytes);
    if expected_sha256.is_some_and(|expected| expected != current_sha256) {
        return Err(
            "Settings file changed on disk. Reload it before saving your formations.".into(),
        );
    }
    let (mut json, had_crc) = decode_settings_json(&current_bytes)?;
    formations_into_settings(&mut json, formations)?;

    let value = blue_marshal::from_json(&json)
        .map_err(|e| format!("Failed to rebuild settings data: {}", e))?;
    let options = blue_marshal::EncodeOptions {
        checksum: had_crc,
        ..Default::default()
    };
    let bytes = blue_marshal::encode(&value, &options)
        .map_err(|e| format!("Failed to encode settings file: {}", e))?;

    let latest_bytes = fs::read(path).map_err(|e| format!("Failed to re-read file: {}", e))?;
    if sha256_hex(&latest_bytes) != current_sha256 {
        return Err(
            "Settings file changed on disk while saving. Reload it before trying again.".into(),
        );
    }
    atomic_write(path, &bytes)?;
    Ok(sha256_hex(&bytes))
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

#[tauri::command]
pub async fn open_formation_editor(
    app: tauri::AppHandle,
    file_path: String,
    entry_name: String,
) -> Result<(), String> {
    // One editor window per settings file; labels only allow [a-zA-Z0-9-/:_]
    let mut hasher = Sha256::new();
    hasher.update(file_path.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    let label = format!("formations-{}", &digest[..12]);

    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.set_focus();
        return Ok(());
    }

    let url = format!(
        "index.html#/formations?path={}&name={}",
        percent_encode(&file_path),
        percent_encode(&entry_name)
    );

    let builder =
        tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App(url.into()))
            .title(format!("Probe Formations — {}", entry_name))
            .inner_size(1150.0, 780.0)
            .min_inner_size(900.0, 620.0)
            .decorations(true);

    // Match the main window's chrome: overlaid traffic lights with a hidden
    // native title on macOS; Windows/Linux draw custom controls in the webview.
    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true);

    builder
        .build()
        .map_err(|e| format!("Failed to open editor window: {}", e))?;

    Ok(())
}

// ── Selective settings copy ──────────────────────────────────────────────
//
// Copies curated groups of settings between files of the same kind instead
// of overwriting the whole file. A group is either whole top-level sections
// or a set of key prefixes inside the "ui" section.

enum GroupRule {
    Sections(&'static [&'static str]),
    UiPrefixes(&'static [&'static str]),
}

// Mirrored in src/lib/copyGroups.ts (COPY_GROUPS): the frontend owns which
// groups are offered per file kind and their defaults; this owns what each
// group id actually matches. Keep the id sets in sync.
fn group_rule(group: &str) -> Option<GroupRule> {
    Some(match group {
        // Account (core_user) groups
        "overview" => GroupRule::Sections(&["overview", "defaultoverview"]),
        "probes" => GroupRule::UiPrefixes(&["probescanning."]),
        "suppress" => GroupRule::Sections(&["suppress"]),
        "audio" => GroupRule::Sections(&["audio"]),
        "camera_graphics" => GroupRule::UiPrefixes(&[
            "camera",
            "spaceMouse",
            "offsetUIwithCamera",
            "invertCameraZoom",
            "advancedCamera",
            "missilesEnabled",
            "turretsEnabled",
            "trailsEnabled",
            "effectsEnabled",
            "explosionEffectsEnabled",
            "gpuParticlesEnabled",
            "droneModelsEnabled",
            "modelSkinsInSpaceEnabled",
            "UI_ASTEROID_",
        ]),
        "market" => GroupRule::UiPrefixes(&[
            "market_",
            "minEdit_market",
            "maxEdit_market",
            "quickbar",
            "contracts_search_",
            "mycontracts_",
            "pricehistorytype",
        ]),
        "slots" => GroupRule::UiPrefixes(&["slotOrder", "linkedWeapons_"]),
        "tabgroups" => GroupRule::Sections(&["tabgroups"]),
        // Typed-text autocomplete and recent-search data; offered for both
        // file kinds — keys missing from a source are simply skipped
        "search_history" => GroupRule::UiPrefixes(&[
            "editHistory",
            "contracts_history",
            "market_searchText",
            "assetsSearch",
        ]),
        // Character (core_char) groups
        "windows" => GroupRule::Sections(&["windows"]),
        "neocom" => GroupRule::UiPrefixes(&["neocomButtonRawData"]),
        "chat" => GroupRule::UiPrefixes(&["chatchannels"]),
        "infopanels" => GroupRule::UiPrefixes(&["InfoPanelModes_"]),
        "dockpanels" => GroupRule::Sections(&["dockPanels"]),
        _ => return None,
    })
}

fn key_matches(key: &str, prefixes: &[&str]) -> bool {
    let stripped = strip_type_prefix(key);
    prefixes.iter().any(|p| stripped.starts_with(p))
}

fn find_section_key(root: &serde_json::Map<String, serde_json::Value>, section: &str) -> String {
    // Sections are usually "bytes:<name>"; fall back to constructing it
    root.keys()
        .find(|k| strip_type_prefix(k) == section)
        .cloned()
        .unwrap_or_else(|| format!("bytes:{}", section))
}

const USER_COPY_GROUPS: [&str; 9] = [
    "overview",
    "probes",
    "suppress",
    "audio",
    "camera_graphics",
    "market",
    "slots",
    "tabgroups",
    "search_history",
];
const CHAR_COPY_GROUPS: [&str; 6] = [
    "windows",
    "neocom",
    "chat",
    "infopanels",
    "dockpanels",
    "search_history",
];

fn copy_selected_groups(
    result: &mut serde_json::Value,
    source: &serde_json::Value,
    included_groups: &[&str],
) -> Result<(), String> {
    let result_map = result
        .as_object_mut()
        .ok_or("Target file has an unexpected structure")?;
    let source_map = source
        .as_object()
        .ok_or("Source file has an unexpected structure")?;

    for group in included_groups {
        let Some(rule) = group_rule(group) else {
            continue;
        };
        match rule {
            GroupRule::Sections(sections) => {
                for section in sections {
                    let key = find_section_key(result_map, section);
                    result_map.remove(&key);
                    let source_key = find_section_key(source_map, section);
                    if let Some(value) = source_map.get(&source_key) {
                        result_map.insert(source_key, value.clone());
                    }
                }
            }
            GroupRule::UiPrefixes(prefixes) => {
                let ui_key = find_section_key(result_map, "ui");
                let empty = serde_json::Map::new();
                let source_ui = source_map
                    .get(&find_section_key(source_map, "ui"))
                    .and_then(|v| v.as_object())
                    .unwrap_or(&empty);

                let result_ui = result_map
                    .entry(ui_key)
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                let Some(result_ui) = result_ui.as_object_mut() else {
                    continue;
                };

                result_ui.retain(|k, _| !key_matches(k, prefixes));
                for (k, v) in source_ui {
                    if key_matches(k, prefixes) {
                        result_ui.insert(k.clone(), v.clone());
                    }
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn copy_settings_selective(
    app: tauri::AppHandle,
    source_path: String,
    target_paths: Vec<String>,
    excluded_groups: Vec<String>,
    backup: Option<bool>,
) -> Result<BatchMutationResult, String> {
    use filetime::FileTime;

    ensure_eve_closed()?;
    let backup = backup.unwrap_or(true);

    let source = Path::new(&source_path);
    let (kind, _) = parse_core_filename(source)?;
    let available_groups: &[&str] = match kind {
        "user" => &USER_COPY_GROUPS,
        "char" => &CHAR_COPY_GROUPS,
        _ => return Err("Unsupported settings file kind".into()),
    };
    let excluded: HashSet<&str> = excluded_groups.iter().map(String::as_str).collect();
    let included: Vec<&str> = available_groups
        .iter()
        .copied()
        .filter(|group| !excluded.contains(group))
        .collect();
    let (source_json, _) = load_settings_json(source)?;
    let mut batch = BatchMutationResult::default();
    let now = FileTime::now();

    for target_path in &target_paths {
        let path = Path::new(target_path);
        let outcome = (|| -> Result<(), String> {
            if *target_path == source_path {
                return Err("Source and target are the same file".into());
            }
            let (target_json, had_crc) = load_settings_json(path)?;
            let mut result = target_json;
            copy_selected_groups(&mut result, &source_json, &included)?;
            let value = blue_marshal::from_json(&result)
                .map_err(|e| format!("Failed to rebuild target settings: {}", e))?;
            let options = blue_marshal::EncodeOptions {
                checksum: had_crc,
                ..Default::default()
            };
            let bytes = blue_marshal::encode(&value, &options)
                .map_err(|e| format!("Failed to encode target settings: {}", e))?;
            if backup {
                auto_backup(path, "pre-selective-copy")?;
            }
            atomic_write(path, &bytes)?;
            let _ = filetime::set_file_mtime(path, now);
            Ok(())
        })();

        match outcome {
            Ok(()) => batch.succeeded.push(target_path.clone()),
            Err(error) => batch.failed.push(MutationFailure {
                path: target_path.clone(),
                error,
            }),
        }
    }

    if !batch.succeeded.is_empty() {
        emit_data_changed(&app);
    }
    Ok(batch)
}

// Current display name (alias if set, otherwise the raw id) for a settings
// file, so secondary windows can stay in sync when aliases change.
#[tauri::command]
pub fn get_entry_display_name(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    let (_, id) = parse_core_filename(path)?;

    let aliases = load_aliases(&app);
    Ok(aliases.get(id).cloned().unwrap_or_else(|| id.to_string()))
}

#[tauri::command]
pub fn read_probe_formations(file_path: String) -> Result<FormationSnapshot, String> {
    read_formations_from_file(Path::new(&file_path))
}

#[tauri::command]
pub fn write_probe_formations(
    app: tauri::AppHandle,
    file_path: String,
    formations: Vec<ProbeFormation>,
    backup: Option<bool>,
    expected_sha256: Option<String>,
) -> Result<FormationWriteResult, String> {
    ensure_eve_closed()?;
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("Settings file does not exist".into());
    }

    let current_bytes = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    if expected_sha256
        .as_deref()
        .is_some_and(|expected| expected != sha256_hex(&current_bytes))
    {
        return Err(
            "Settings file changed on disk. Reload it before saving your formations.".into(),
        );
    }
    validate_formations(&formations)?;
    if backup.unwrap_or(true) {
        auto_backup(path, "pre-formation-edit")?;
    }
    let file_sha256 = write_formations_to_file(path, &formations, expected_sha256.as_deref())?;

    emit_data_changed(&app);
    Ok(FormationWriteResult { file_sha256 })
}

#[tauri::command]
pub fn export_probe_formation(
    export_path: String,
    formation: ProbeFormation,
) -> Result<(), String> {
    validate_formations(std::slice::from_ref(&formation))?;
    let portable = PortableFormation {
        schema_version: 1,
        name: formation.name,
        probes: formation
            .probes
            .into_iter()
            .map(|p| PortableProbe {
                x_km: p.x / 1000.0,
                y_km: p.y / 1000.0,
                z_km: p.z / 1000.0,
                range_au: p.range / AU_METERS,
            })
            .collect(),
    };
    let data = serde_json::to_vec_pretty(&portable)
        .map_err(|e| format!("Failed to serialize formation: {}", e))?;
    atomic_write(Path::new(&export_path), &data)
}

#[tauri::command]
pub fn export_probe_formations(
    export_path: String,
    formations: Vec<ProbeFormation>,
) -> Result<(), String> {
    validate_formations(&formations)?;
    let pack = PortableFormationPack {
        schema_version: 1,
        formations: formations
            .into_iter()
            .map(|formation| PortableFormation {
                schema_version: 1,
                name: formation.name,
                probes: formation
                    .probes
                    .into_iter()
                    .map(|probe| PortableProbe {
                        x_km: probe.x / 1000.0,
                        y_km: probe.y / 1000.0,
                        z_km: probe.z / 1000.0,
                        range_au: probe.range / AU_METERS,
                    })
                    .collect(),
            })
            .collect(),
    };
    let data = serde_json::to_vec_pretty(&pack)
        .map_err(|e| format!("Failed to serialize formation pack: {}", e))?;
    atomic_write(Path::new(&export_path), &data)
}

#[tauri::command]
pub fn import_probe_formation(import_path: String) -> Result<Vec<ProbeFormation>, String> {
    let path = Path::new(&import_path);
    if fs::metadata(path)
        .map_err(|e| format!("Failed to open formation: {}", e))?
        .len()
        > 1024 * 1024
    {
        return Err("Formation JSON is larger than 1 MiB".into());
    }
    let data = fs::read(path).map_err(|e| format!("Failed to read formation: {}", e))?;
    let value: serde_json::Value =
        serde_json::from_slice(&data).map_err(|e| format!("Invalid formation JSON: {}", e))?;
    let portable_formations = if value.get("formations").is_some() {
        let pack: PortableFormationPack =
            serde_json::from_value(value).map_err(|e| format!("Invalid formation pack: {}", e))?;
        if pack.schema_version != 1 {
            return Err(format!(
                "Unsupported formation pack schema version {}",
                pack.schema_version
            ));
        }
        pack.formations
    } else {
        vec![serde_json::from_value(value).map_err(|e| format!("Invalid formation JSON: {}", e))?]
    };
    let formations: Vec<ProbeFormation> = portable_formations
        .into_iter()
        .enumerate()
        .map(|(id, portable)| {
            if portable.schema_version != 1 {
                return Err(format!(
                    "Unsupported formation schema version {}",
                    portable.schema_version
                ));
            }
            Ok(ProbeFormation {
                id: id as i64,
                name: portable.name,
                probes: portable
                    .probes
                    .into_iter()
                    .map(|p| FormationProbe {
                        x: p.x_km * 1000.0,
                        y: p.y_km * 1000.0,
                        z: p.z_km * 1000.0,
                        range: p.range_au * AU_METERS,
                    })
                    .collect(),
            })
        })
        .collect::<Result<_, String>>()?;
    validate_formations(&formations)?;
    Ok(formations)
}

#[cfg(test)]
mod formation_tests {
    use super::*;
    use serde_json::json;

    fn sample_formations() -> Vec<ProbeFormation> {
        vec![ProbeFormation {
            id: 0,
            name: "pinpoint".to_string(),
            probes: vec![
                FormationProbe {
                    x: 250_000.0,
                    y: 0.0,
                    z: 0.0,
                    range: 37_399_467_675.0,
                },
                FormationProbe {
                    x: 0.0,
                    y: -500_000.0,
                    z: 0.0,
                    range: 37_399_467_675.0,
                },
            ],
        }]
    }

    #[test]
    fn backup_identity_uses_the_unique_file_path() {
        let first = Path::new("server/settings_Default/backups/pre_user_1_10.bak");
        let second = Path::new("server/settings_Default/backups/pre_user_2_10.bak");
        assert_ne!(backup_entry_id(first), backup_entry_id(second));
    }

    #[test]
    fn accepts_combat_probe_64_au_range() {
        let formation = ProbeFormation {
            id: 0,
            name: "combat deep spread".to_string(),
            probes: vec![FormationProbe {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                range: 64.0 * AU_METERS,
            }],
        };

        assert!(validate_formations(&[formation]).is_ok());
    }

    #[test]
    fn formations_round_trip_through_marshal() {
        let dir = std::env::temp_dir().join("eve-wrench-formation-test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("core_user_12345.dat");

        // Minimal settings file with an unrelated key that must survive and a
        // client-internal temp formation that must be hidden but preserved
        let initial = json!({
            "bytes:ui": {
                "bytes:missilesEnabled": {"tuple": ["long:134280801871504062", true]},
                "bytes:probescanning.selectedFormationID": {"tuple": ["long:134280801860500867", 7]},
                "bytes:probescanning.customFormations": {"tuple": ["long:134280801871504062", {
                    "int:-4": {"tuple": ["bytes:tempFormation", []]},
                }]},
            }
        });
        let value = blue_marshal::from_json(&initial).unwrap();
        let bytes = blue_marshal::encode(&value, &blue_marshal::EncodeOptions::default()).unwrap();
        fs::write(&path, bytes).unwrap();

        assert!(read_formations_from_file(&path)
            .unwrap()
            .formations
            .is_empty());

        let formations = sample_formations();
        write_formations_to_file(&path, &formations, None).unwrap();

        let read_back = read_formations_from_file(&path).unwrap().formations;
        assert_eq!(read_back.len(), 1);
        assert_eq!(read_back[0].name, "pinpoint");
        assert_eq!(read_back[0].probes.len(), 2);
        assert_eq!(read_back[0].probes[0].x, 250_000.0);
        assert_eq!(read_back[0].probes[1].y, -500_000.0);

        // Unrelated key untouched, selected id repaired to a valid one
        let (json, _) = load_settings_json(&path).unwrap();
        let ui = json.get("bytes:ui").unwrap();
        assert_eq!(
            ui.get("bytes:missilesEnabled")
                .unwrap()
                .get("tuple")
                .unwrap()[1],
            json!(true)
        );
        assert_eq!(
            ui.get(SELECTED_FORMATION_KEY)
                .unwrap()
                .get("tuple")
                .unwrap()[1],
            json!(0)
        );

        // Client-internal temp formation preserved alongside user formations
        let stored = ui.get(FORMATIONS_KEY).unwrap().get("tuple").unwrap()[1]
            .as_object()
            .unwrap();
        assert!(stored.contains_key("int:-4"));
        assert!(stored.contains_key("int:0"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn formation_pack_imports_multiple_formations() {
        let dir = std::env::temp_dir().join("eve-wrench-formation-pack-test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("formations.json");
        let pack = json!({
            "schema_version": 1,
            "formations": [
                {
                    "schema_version": 1,
                    "name": "Alpha",
                    "probes": [{"x_km": 100.0, "y_km": 0.0, "z_km": 0.0, "range_au": 0.25}]
                },
                {
                    "schema_version": 1,
                    "name": "Bravo",
                    "probes": [{"x_km": 0.0, "y_km": 200.0, "z_km": 0.0, "range_au": 0.5}]
                }
            ]
        });
        fs::write(&path, serde_json::to_vec(&pack).unwrap()).unwrap();

        let imported = import_probe_formation(path.to_string_lossy().into_owned()).unwrap();
        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].name, "Alpha");
        assert_eq!(imported[1].probes[0].y, 200_000.0);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn selective_copy_copies_only_selected_groups() {
        let source = json!({
            "bytes:overview": {
                "bytes:shipLabels": {"tuple": ["long:1", "utf8:source-labels"]},
            },
            "bytes:ui": {
                "bytes:probescanning.customFormations": {"tuple": ["long:1", {"int:0": {"tuple": ["utf8:src", []]}}]},
                "bytes:slotOrder": {"tuple": ["long:1", "utf8:source-slots"]},
                "bytes:someUnmappedKey": {"tuple": ["long:1", "utf8:source-unmapped"]},
            },
        });
        let target = json!({
            "bytes:overview": {
                "bytes:shipLabels": {"tuple": ["long:2", "utf8:target-labels"]},
            },
            "bytes:ui": {
                "bytes:probescanning.customFormations": {"tuple": ["long:2", {}]},
                "bytes:slotOrder": {"tuple": ["long:2", "utf8:target-slots"]},
                "bytes:linkedWeapons_groupsDict": {"tuple": ["long:2", "utf8:target-links"]},
                "bytes:someUnmappedKey": {"tuple": ["long:2", "utf8:target-unmapped"]},
            },
        });

        let mut result = target.clone();
        copy_selected_groups(&mut result, &source, &["overview", "probes"]).unwrap();

        let ui = result.get("bytes:ui").unwrap();
        // Not excluded: mapped and unmapped keys alike come from source
        assert_eq!(
            result["bytes:overview"]["bytes:shipLabels"]["tuple"][1],
            json!("utf8:source-labels")
        );
        assert!(ui["bytes:probescanning.customFormations"]["tuple"][1]
            .as_object()
            .unwrap()
            .contains_key("int:0"));
        assert_eq!(
            ui["bytes:someUnmappedKey"]["tuple"][1],
            json!("utf8:target-unmapped")
        );
        // Excluded: slot layout keeps exactly what the target had, including
        // keys the source does not have at all
        assert_eq!(
            ui["bytes:slotOrder"]["tuple"][1],
            json!("utf8:target-slots")
        );
        assert_eq!(
            ui["bytes:linkedWeapons_groupsDict"]["tuple"][1],
            json!("utf8:target-links")
        );
    }
}

#[tauri::command]
pub fn create_backup(
    app: tauri::AppHandle,
    source_path: String,
    backup_name: String,
) -> Result<BackupEntry, String> {
    let source = PathBuf::from(&source_path);

    if !source.exists() {
        return Err("Source file does not exist".into());
    }

    let (kind_str, id) = parse_core_filename(&source)?;
    let kind = match kind_str {
        "user" => SettingsKind::User,
        "char" => SettingsKind::Char,
        _ => return Err("Unknown settings type".into()),
    };
    let id = id.to_string();
    let (server, profile) = settings_path_context(&source)?;

    let (dest, timestamp) = auto_backup(&source, &backup_name)?;
    let path = dest.to_string_lossy().into_owned();

    let entry = BackupEntry {
        id: backup_entry_id(&dest),
        display_name: backup_name.clone(),
        name: backup_name,
        path,
        timestamp,
        kind,
        original_id: id.to_string(),
        original_name: None,
        server,
        profile,
        relative_time: format_relative_time(timestamp),
    };

    emit_data_changed(&app);
    Ok(entry)
}

#[tauri::command]
pub fn delete_backup(app: tauri::AppHandle, backup_path: String) -> Result<(), String> {
    let path = PathBuf::from(&backup_path);

    if !path.exists() {
        return Err("Backup file not found".into());
    }

    fs::remove_file(path).map_err(|e| e.to_string())?;
    emit_data_changed(&app);
    Ok(())
}

// Deletes several backups at once, returning how many were removed. Missing
// files are skipped rather than failing the whole batch.
#[tauri::command]
pub fn delete_backups(app: tauri::AppHandle, backup_paths: Vec<String>) -> Result<usize, String> {
    let mut deleted = 0;
    for backup_path in backup_paths {
        let path = PathBuf::from(&backup_path);
        if path.exists() && fs::remove_file(&path).is_ok() {
            deleted += 1;
        }
    }
    if deleted > 0 {
        emit_data_changed(&app);
    }
    Ok(deleted)
}

#[tauri::command]
pub fn copy_settings(
    app: tauri::AppHandle,
    source_path: String,
    target_paths: Vec<String>,
    backup: Option<bool>,
) -> Result<BatchMutationResult, String> {
    use filetime::FileTime;

    ensure_eve_closed()?;
    let src = PathBuf::from(&source_path);

    if !src.exists() {
        return Err("Source file not found".into());
    }

    let bytes = fs::read(&src).map_err(|e| format!("Failed to read source file: {}", e))?;
    let mut batch = BatchMutationResult::default();
    let now = FileTime::now();
    let backup = backup.unwrap_or(true);

    for target_path in target_paths {
        let dest = PathBuf::from(&target_path);
        let outcome = (|| -> Result<(), String> {
            if src == dest {
                return Err("Source and target are the same file".into());
            }
            if backup {
                auto_backup(&dest, "pre-restore")?;
            }
            atomic_write(&dest, &bytes)?;
            let _ = filetime::set_file_mtime(&dest, now);
            Ok(())
        })();

        match outcome {
            Ok(()) => batch.succeeded.push(target_path),
            Err(error) => batch.failed.push(MutationFailure {
                path: target_path,
                error,
            }),
        }
    }

    if !batch.succeeded.is_empty() {
        emit_data_changed(&app);
    }
    Ok(batch)
}

#[tauri::command]
pub fn set_alias(
    app: tauri::AppHandle,
    account_id: String,
    alias: Option<String>,
) -> Result<(), String> {
    let mut aliases = load_aliases(&app);

    match alias {
        Some(a) if !a.trim().is_empty() => {
            aliases.insert(account_id, a.trim().to_string());
        }
        _ => {
            aliases.remove(&account_id);
        }
    }

    save_aliases(&app, &aliases)?;
    emit_data_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn set_brackets_always_show(
    app: tauri::AppHandle,
    server_path: String,
    enabled: bool,
) -> Result<(), String> {
    ensure_eve_closed()?;
    let path = PathBuf::from(&server_path);
    write_brackets_setting(&path, enabled)?;
    emit_data_changed(&app);
    Ok(())
}

// ── Export / Import ──────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestFileEntry {
    pub relative_path: String,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportManifest {
    pub app_version: String,
    pub timestamp: u64,
    pub files: Vec<ManifestFileEntry>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ExportResult {
    pub file_count: usize,
    pub path: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ImportFileInfo {
    pub relative_path: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ImportConflictInfo {
    pub relative_path: String,
    pub local_modified: u64,
    pub archive_checksum: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ImportAnalysis {
    pub new_files: Vec<ImportFileInfo>,
    pub conflicts: Vec<ImportConflictInfo>,
    pub unchanged: Vec<ImportFileInfo>,
    pub aliases_conflict: bool,
    pub total_files: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct ImportResultInfo {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub backed_up_count: usize,
}

#[derive(Debug)]
struct ImportPlanItem {
    entry: ManifestFileEntry,
    relative_path: String,
    target_path: PathBuf,
    existed: bool,
}

#[derive(Debug)]
struct ImportSafetyBackup {
    target_path: PathBuf,
    backup_path: PathBuf,
    keep_after_success: bool,
}

fn sha256_of_bytes(data: &[u8]) -> String {
    sha256_hex(data)
}

const MAX_IMPORT_FILES: usize = 10_000;
const MAX_IMPORT_FILE_BYTES: u64 = 32 * 1024 * 1024;

fn normalize_manifest_path(raw: &str) -> Result<(String, PathBuf), String> {
    let normalized = raw.replace('\\', "/");
    if normalized == "aliases.json" {
        return Ok((normalized, PathBuf::from("aliases.json")));
    }
    let mut parts = Vec::new();
    for component in Path::new(&normalized).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            _ => return Err(format!("Unsafe archive path: {}", raw)),
        }
    }
    let valid = match parts.as_slice() {
        [_, profile, file]
            if profile.starts_with("settings_")
                && (file == "prefs.ini"
                    || (file.starts_with("core_") && file.ends_with(".dat"))) =>
        {
            true
        }
        [_, profile, backups, file]
            if profile.starts_with("settings_")
                && backups == "backups"
                && file.ends_with(".bak") =>
        {
            true
        }
        _ => false,
    };
    if !valid {
        return Err(format!("Unsupported archive path: {}", raw));
    }
    Ok((parts.join("/"), parts.iter().collect()))
}

fn validate_manifest(manifest: &ExportManifest) -> Result<(), String> {
    if manifest.files.len() > MAX_IMPORT_FILES {
        return Err(format!(
            "Archive contains more than {} files",
            MAX_IMPORT_FILES
        ));
    }
    let mut paths = HashSet::new();
    for entry in &manifest.files {
        let (normalized, _) = normalize_manifest_path(&entry.relative_path)?;
        if !paths.insert(normalized) {
            return Err(format!(
                "Archive manifest contains duplicate path {}",
                entry.relative_path
            ));
        }
        if entry.sha256.len() != 64 || !entry.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "Archive manifest has an invalid checksum for {}",
                entry.relative_path
            ));
        }
    }
    Ok(())
}

fn read_verified_archive_entry<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    entry: &ManifestFileEntry,
) -> Result<Vec<u8>, String> {
    let mut file = archive
        .by_name(&entry.relative_path)
        .map_err(|_| format!("Archive is missing {}", entry.relative_path))?;
    if file.size() > MAX_IMPORT_FILE_BYTES {
        return Err(format!(
            "{} is larger than the {} MiB import limit",
            entry.relative_path,
            MAX_IMPORT_FILE_BYTES / 1024 / 1024
        ));
    }
    let mut data = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read {}: {}", entry.relative_path, e))?;
    if sha256_hex(&data) != entry.sha256.to_ascii_lowercase() {
        return Err(format!(
            "Checksum mismatch for {}. The archive may be damaged or modified.",
            entry.relative_path
        ));
    }
    Ok(data)
}

fn sha256_of_file(path: &Path) -> Result<String, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    Ok(sha256_of_bytes(&data))
}

fn create_temporary_import_backup(path: &Path, index: usize) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("settings");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let backup_path = parent.join(format!(
        ".{}.eve-wrench-import.{}.{}.{}.rollback",
        file_name,
        std::process::id(),
        nonce,
        index
    ));
    fs::copy(path, &backup_path).map_err(|error| {
        format!(
            "Failed to create a safety copy of {}: {}",
            path.display(),
            error
        )
    })?;
    Ok(backup_path)
}

fn restore_import_changes(
    written: &[&ImportPlanItem],
    safety_backups: &[ImportSafetyBackup],
) -> Vec<String> {
    let backups: HashMap<&Path, &Path> = safety_backups
        .iter()
        .map(|backup| (backup.target_path.as_path(), backup.backup_path.as_path()))
        .collect();
    let mut failures = Vec::new();

    for item in written.iter().rev() {
        if item.existed {
            let Some(backup_path) = backups.get(item.target_path.as_path()) else {
                failures.push(format!(
                    "No rollback copy was available for {}",
                    item.relative_path
                ));
                continue;
            };
            match fs::read(backup_path)
                .map_err(|error| error.to_string())
                .and_then(|data| atomic_write(&item.target_path, &data))
            {
                Ok(()) => {}
                Err(error) => failures.push(format!(
                    "Could not restore {}: {}",
                    item.relative_path, error
                )),
            }
        } else if item.target_path.exists() {
            if let Err(error) = fs::remove_file(&item.target_path) {
                failures.push(format!(
                    "Could not remove new file {}: {}",
                    item.relative_path, error
                ));
            }
        }
    }

    failures
}

fn remove_import_safety_backups(backups: &[ImportSafetyBackup], include_permanent: bool) {
    for backup in backups {
        if include_permanent || !backup.keep_after_success {
            let _ = fs::remove_file(&backup.backup_path);
        }
    }
}

fn collect_exportable_files(eve_root: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let mut files: Vec<(PathBuf, String)> = Vec::new();

    let server_dirs = fs::read_dir(eve_root).map_err(|e| e.to_string())?;

    for server_entry in server_dirs.flatten() {
        let server_path = server_entry.path();
        if !server_path.is_dir() {
            continue;
        }

        let profile_dirs = match fs::read_dir(&server_path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for profile_entry in profile_dirs.flatten() {
            let profile_path = profile_entry.path();
            if !profile_path.is_dir() {
                continue;
            }

            let dir_name = profile_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !dir_name.starts_with("settings_") {
                continue;
            }

            // Collect .dat files
            if let Ok(entries) = fs::read_dir(&profile_path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if fname.ends_with(".dat") && fname.starts_with("core_") {
                            let rel = p
                                .strip_prefix(eve_root)
                                .map_err(|e| e.to_string())?
                                .to_string_lossy()
                                .into_owned();
                            files.push((p, rel));
                        } else if fname == "prefs.ini" {
                            let rel = p
                                .strip_prefix(eve_root)
                                .map_err(|e| e.to_string())?
                                .to_string_lossy()
                                .into_owned();
                            files.push((p, rel));
                        }
                    }
                }
            }

            // Collect backup files
            let backup_dir = profile_path.join("backups");
            if backup_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&backup_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() {
                            let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            if fname.ends_with(".bak") {
                                let rel = p
                                    .strip_prefix(eve_root)
                                    .map_err(|e| e.to_string())?
                                    .to_string_lossy()
                                    .into_owned();
                                files.push((p, rel));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(files)
}

#[tauri::command]
pub fn export_settings(
    app: tauri::AppHandle,
    custom_eve_path: Option<String>,
    export_path: String,
) -> Result<ExportResult, String> {
    let eve_root =
        eve_settings_root(custom_eve_path.as_deref()).ok_or("EVE settings directory not found")?;

    if !eve_root.exists() {
        return Err("EVE settings directory does not exist".into());
    }

    let exportable_files = collect_exportable_files(&eve_root)?;

    let dest = PathBuf::from(&export_path);
    let file = fs::File::create(&dest).map_err(|e| format!("Failed to create zip: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut manifest_files: Vec<ManifestFileEntry> = Vec::new();

    for (abs_path, rel_path) in &exportable_files {
        let data = fs::read(abs_path)
            .map_err(|e| format!("Failed to read {}: {}", abs_path.display(), e))?;
        let checksum = sha256_of_bytes(&data);

        zip.start_file(rel_path, options)
            .map_err(|e| format!("Failed to add to zip: {}", e))?;
        zip.write_all(&data)
            .map_err(|e| format!("Failed to write to zip: {}", e))?;

        manifest_files.push(ManifestFileEntry {
            relative_path: rel_path.clone(),
            sha256: checksum,
        });
    }

    // Add aliases.json if it exists
    let aliases_path = aliases_file(&app)?;
    if aliases_path.exists() {
        let data = fs::read(&aliases_path).map_err(|e| format!("Failed to read aliases: {}", e))?;
        let checksum = sha256_of_bytes(&data);

        zip.start_file("aliases.json", options)
            .map_err(|e| format!("Failed to add aliases to zip: {}", e))?;
        zip.write_all(&data)
            .map_err(|e| format!("Failed to write aliases to zip: {}", e))?;

        manifest_files.push(ManifestFileEntry {
            relative_path: "aliases.json".to_string(),
            sha256: checksum,
        });
    }

    // Create and add manifest
    let version = env!("CARGO_PKG_VERSION").to_string();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let manifest = ExportManifest {
        app_version: version,
        timestamp,
        files: manifest_files,
    };

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;

    zip.start_file("manifest.json", options)
        .map_err(|e| format!("Failed to add manifest: {}", e))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    zip.finish()
        .map_err(|e| format!("Failed to finalize zip: {}", e))?;

    let file_count = manifest.files.len();
    Ok(ExportResult {
        file_count,
        path: export_path,
    })
}

#[tauri::command]
pub fn analyze_import(
    app: tauri::AppHandle,
    import_path: String,
    custom_eve_path: Option<String>,
) -> Result<ImportAnalysis, String> {
    let eve_root =
        eve_settings_root(custom_eve_path.as_deref()).ok_or("EVE settings directory not found")?;

    let file = fs::File::open(&import_path).map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid zip file: {}", e))?;

    // Read manifest
    let manifest: ExportManifest = {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .map_err(|_| "No manifest.json found in archive - not a valid EVE Wrench export")?;
        if manifest_file.size() > 2 * 1024 * 1024 {
            return Err("Archive manifest is larger than 2 MiB".into());
        }
        let mut content = String::new();
        manifest_file
            .read_to_string(&mut content)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Invalid manifest: {}", e))?
    };
    validate_manifest(&manifest)?;

    let mut new_files: Vec<ImportFileInfo> = Vec::new();
    let mut conflicts: Vec<ImportConflictInfo> = Vec::new();
    let mut unchanged: Vec<ImportFileInfo> = Vec::new();
    let mut aliases_conflict = false;

    let aliases_path = aliases_file(&app)?;

    for entry in &manifest.files {
        let (rel, safe_relative_path) = normalize_manifest_path(&entry.relative_path)?;
        let _ = read_verified_archive_entry(&mut archive, entry)?;

        if rel == "aliases.json" {
            if aliases_path.exists() {
                let local_checksum = sha256_of_file(&aliases_path)?;
                if !local_checksum.eq_ignore_ascii_case(&entry.sha256) {
                    aliases_conflict = true;
                    conflicts.push(ImportConflictInfo {
                        relative_path: rel.clone(),
                        local_modified: fs::metadata(&aliases_path)
                            .and_then(|m| m.modified())
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                        archive_checksum: entry.sha256.clone(),
                    });
                } else {
                    unchanged.push(ImportFileInfo {
                        relative_path: rel.clone(),
                    });
                }
            } else {
                new_files.push(ImportFileInfo {
                    relative_path: rel.clone(),
                });
            }
            continue;
        }

        let local_path = eve_root.join(safe_relative_path);
        if local_path.exists() {
            let local_checksum = sha256_of_file(&local_path)?;
            if !local_checksum.eq_ignore_ascii_case(&entry.sha256) {
                let local_modified = fs::metadata(&local_path)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                conflicts.push(ImportConflictInfo {
                    relative_path: rel.clone(),
                    local_modified,
                    archive_checksum: entry.sha256.clone(),
                });
            } else {
                unchanged.push(ImportFileInfo {
                    relative_path: rel.clone(),
                });
            }
        } else {
            new_files.push(ImportFileInfo { relative_path: rel });
        }
    }

    let total_files = manifest.files.len();

    Ok(ImportAnalysis {
        new_files,
        conflicts,
        unchanged,
        aliases_conflict,
        total_files,
    })
}

#[tauri::command]
pub fn execute_import(
    app: tauri::AppHandle,
    import_path: String,
    custom_eve_path: Option<String>,
    overwrite_paths: Vec<String>,
) -> Result<ImportResultInfo, String> {
    ensure_eve_closed()?;
    let eve_root =
        eve_settings_root(custom_eve_path.as_deref()).ok_or("EVE settings directory not found")?;

    let file = fs::File::open(&import_path).map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid zip file: {}", e))?;

    // Read manifest
    let manifest: ExportManifest = {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .map_err(|_| "No manifest.json found in archive")?;
        if manifest_file.size() > 2 * 1024 * 1024 {
            return Err("Archive manifest is larger than 2 MiB".into());
        }
        let mut content = String::new();
        manifest_file
            .read_to_string(&mut content)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Invalid manifest: {}", e))?
    };
    validate_manifest(&manifest)?;

    let aliases_path = aliases_file(&app)?;
    let overwrite_set: HashSet<String> = overwrite_paths
        .iter()
        .filter_map(|path| {
            normalize_manifest_path(path)
                .ok()
                .map(|(normalized, _)| normalized)
        })
        .collect();

    // Phase one: verify every archive payload and build the complete plan.
    // No local setting is changed until this succeeds for the whole archive.
    let mut plan = Vec::new();
    let mut skipped_count = 0usize;
    for entry in &manifest.files {
        let (relative_path, safe_relative_path) = normalize_manifest_path(&entry.relative_path)?;
        let _ = read_verified_archive_entry(&mut archive, entry)?;
        let target_path = if relative_path == "aliases.json" {
            aliases_path.clone()
        } else {
            eve_root.join(safe_relative_path)
        };
        let existed = target_path.exists();
        if existed {
            let local_checksum = sha256_of_file(&target_path)?;
            if local_checksum.eq_ignore_ascii_case(&entry.sha256)
                || !overwrite_set.contains(&relative_path)
            {
                skipped_count += 1;
                continue;
            }
        }
        plan.push(ImportPlanItem {
            entry: entry.clone(),
            relative_path,
            target_path,
            existed,
        });
    }

    // Phase two: create every required safety copy before the first write.
    // Core settings keep their normal user-visible pre-import backup. Other
    // files receive a temporary rollback copy for the duration of the import.
    let mut safety_backups = Vec::new();
    let mut backed_up_count = 0usize;
    for (index, item) in plan.iter().enumerate().filter(|(_, item)| item.existed) {
        let backup = if parse_core_filename(&item.target_path).is_ok() {
            let (backup_path, _) = auto_backup(&item.target_path, "pre-import").map_err(
                |error| {
                    remove_import_safety_backups(&safety_backups, false);
                    format!(
                        "Import stopped before making changes because {} could not be backed up: {}",
                        item.relative_path, error
                    )
                },
            )?;
            backed_up_count += 1;
            ImportSafetyBackup {
                target_path: item.target_path.clone(),
                backup_path,
                keep_after_success: true,
            }
        } else {
            let backup_path =
                create_temporary_import_backup(&item.target_path, index).map_err(|error| {
                    remove_import_safety_backups(&safety_backups, false);
                    format!(
                        "Import stopped before making changes because {} could not be secured: {}",
                        item.relative_path, error
                    )
                })?;
            ImportSafetyBackup {
                target_path: item.target_path.clone(),
                backup_path,
                keep_after_success: false,
            }
        };
        safety_backups.push(backup);
    }

    // Phase three: perform the writes. If any one fails, restore everything
    // already written and report any rollback problem explicitly.
    let mut written: Vec<&ImportPlanItem> = Vec::new();
    for item in &plan {
        let result = (|| -> Result<(), String> {
            let data = read_verified_archive_entry(&mut archive, &item.entry)?;
            if let Some(parent) = item.target_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("Failed to create directory: {}", error))?;
            }
            atomic_write(&item.target_path, &data)
                .map_err(|error| format!("Failed to write {}: {}", item.relative_path, error))
        })();

        if let Err(error) = result {
            let rollback_failures = restore_import_changes(&written, &safety_backups);
            if rollback_failures.is_empty() {
                remove_import_safety_backups(&safety_backups, false);
            }
            emit_data_changed(&app);
            let rollback_summary = if rollback_failures.is_empty() {
                "All earlier writes were rolled back.".to_string()
            } else {
                format!("Rollback also reported: {}", rollback_failures.join("; "))
            };
            return Err(format!("{} {}", error, rollback_summary));
        }
        written.push(item);
    }

    remove_import_safety_backups(&safety_backups, false);
    emit_data_changed(&app);

    Ok(ImportResultInfo {
        imported_count: plan.len(),
        skipped_count,
        backed_up_count,
    })
}
