use crate::esi::CLIENT;
use serde::Serialize;

const GITHUB_REPO: &str = "eve-wrench/eve-wrench-app";

#[derive(Serialize, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
}

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

// Stable builds only look at the latest stable release (GitHub's
// /releases/latest never returns pre-releases). Preview builds scan the full
// release list so testers are notified about newer previews AND the next
// stable release.
#[tauri::command]
pub async fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let include_prereleases = is_preview_version(current_version);

    let url = if include_prereleases {
        format!(
            "https://api.github.com/repos/{}/releases?per_page=15",
            GITHUB_REPO
        )
    } else {
        format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_REPO
        )
    };

    let response = CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release: {}", e))?;

    let releases: Vec<&serde_json::Value> = match payload.as_array() {
        Some(list) => list.iter().collect(),
        None => vec![&payload],
    };

    let mut best: Option<UpdateInfo> = None;
    for release in releases {
        if release["draft"].as_bool().unwrap_or(false) {
            continue;
        }
        let version = release["tag_name"]
            .as_str()
            .unwrap_or("")
            .trim_start_matches('v');

        let newer_than_current = is_newer_version(version, current_version);
        let newer_than_best = best
            .as_ref()
            .is_none_or(|b| is_newer_version(version, &b.latest_version));
        if newer_than_current && newer_than_best {
            best = Some(UpdateInfo {
                current_version: current_version.to_string(),
                latest_version: version.to_string(),
                release_url: release["html_url"].as_str().unwrap_or("").to_string(),
                release_notes: release["body"].as_str().unwrap_or("").to_string(),
            });
        }
    }

    Ok(best)
}

// Semver-ordered comparison: numeric cores first; on equal cores a stable
// release beats any pre-release of it, and pre-releases compare by their
// numeric identifiers ("preview.2" > "preview.1").
fn is_newer_version(latest: &str, current: &str) -> bool {
    fn split(v: &str) -> (Vec<u32>, Option<Vec<u32>>) {
        let mut parts = v.splitn(2, '-');
        let core = parts.next().unwrap_or("");
        let pre = parts.next();
        let nums = |s: &str| {
            s.split('.')
                .filter_map(|p| p.parse().ok())
                .collect::<Vec<u32>>()
        };
        (nums(core), pre.map(nums))
    }

    let (latest_core, latest_pre) = split(latest);
    let (current_core, current_pre) = split(current);

    for i in 0..latest_core.len().max(current_core.len()) {
        let l = latest_core.get(i).copied().unwrap_or(0);
        let c = current_core.get(i).copied().unwrap_or(0);
        if l != c {
            return l > c;
        }
    }

    match (latest_pre, current_pre) {
        (None, Some(_)) => true, // 0.3.0 beats 0.3.0-preview.1
        (None, None) | (Some(_), None) => false,
        (Some(l), Some(c)) => {
            for i in 0..l.len().max(c.len()) {
                let a = l.get(i).copied().unwrap_or(0);
                let b = c.get(i).copied().unwrap_or(0);
                if a != b {
                    return a > b;
                }
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(is_newer_version("0.3.0", "0.2.1"));
        assert!(!is_newer_version("0.2.1", "0.2.1"));
        assert!(!is_newer_version("0.2.0", "0.2.1"));
        // Pre-release ordering
        assert!(!is_newer_version("0.2.1", "0.3.0-preview.1"));
        assert!(is_newer_version("0.3.0", "0.3.0-preview.1"));
        assert!(!is_newer_version("0.3.0-preview.1", "0.3.0"));
        assert!(is_newer_version("0.3.0-preview.2", "0.3.0-preview.1"));
        assert!(!is_newer_version("0.3.0-preview.1", "0.3.0-preview.2"));
        assert!(is_newer_version("0.3.0-preview.1", "0.2.1"));
    }

    #[test]
    fn preview_detection() {
        assert!(is_preview_version("0.3.0-preview.1"));
        assert!(is_preview_version("1.0.0-rc.2"));
        assert!(!is_preview_version("0.2.1"));
    }
}
