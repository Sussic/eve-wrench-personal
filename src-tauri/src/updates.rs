use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct AppInfo {
    pub version: String,
    pub preview: bool,
}

// A semver pre-release suffix ("0.3.0-preview.1") marks preview builds.
fn is_preview_version(version: &str) -> bool {
    version.contains('-')
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    AppInfo {
        preview: is_preview_version(&version),
        version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_detection() {
        assert!(is_preview_version("0.3.0-preview.1"));
        assert!(is_preview_version("1.0.0-rc.2"));
        assert!(!is_preview_version("0.2.1"));
    }
}
