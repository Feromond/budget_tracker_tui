use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

pub fn check_for_updates() -> Option<String> {
    let current_version = env!("CARGO_PKG_VERSION");
    
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(3)))
        .build()
        .new_agent();

    let response = agent.get("https://api.github.com/repos/Feromond/budget_tracker_tui/releases/latest")
        .header("User-Agent", "budget_tracker_tui_update_checker")
        .call();

    if let Ok(resp) = response {
        if let Ok(release) = resp.into_body().read_json::<Release>() {
            let remote_version_str = release.tag_name.trim_start_matches('v');
            
            // Basic semver parsing and comparison
            if let (Ok(current), Ok(remote)) = (semver::Version::parse(current_version), semver::Version::parse(remote_version_str)) {
                if remote > current {
                     return Some(release.tag_name);
                }
            }
        }
    }
    
    None
}

