use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use adblock::{
    Engine,
    lists::{FilterSet, ParseOptions},
    request::Request,
};
use url::Url;

const FALLBACK_RULES: &str = r#"
||doubleclick.net^
||googlesyndication.com^
||google-analytics.com^
||adservice.google.com^
||connect.facebook.net^$third-party
||ads-twitter.com^
"#;

pub struct Shields {
    engine: Engine,
    disabled_sites: HashSet<String>,
    blocked_by_tab: HashMap<u64, u64>,
}

impl Shields {
    pub fn load(filter_dir: &Path, disabled_sites: HashSet<String>) -> Self {
        let mut set = FilterSet::new(false);
        set.add_filter_list(FALLBACK_RULES.to_string(), ParseOptions::default());
        for name in ["easylist.txt", "easyprivacy.txt"] {
            if let Ok(text) = fs::read_to_string(filter_dir.join(name)) {
                set.add_filter_list(text, ParseOptions::default());
            }
        }
        Self {
            engine: Engine::new_with_filter_set(set),
            disabled_sites,
            blocked_by_tab: HashMap::new(),
        }
    }

    pub fn should_block(&mut self, tab_id: u64, page_url: &str, request_url: &str) -> bool {
        if !self.enabled_for_url(page_url) {
            return false;
        }
        let Ok(request) = Request::new(request_url, page_url, "other", "") else {
            return false;
        };
        let blocked = self.engine.check_network_request(&request).should_block();
        if blocked {
            *self.blocked_by_tab.entry(tab_id).or_default() += 1;
        }
        blocked
    }

    pub fn blocked_count(&self, tab_id: u64) -> u64 {
        self.blocked_by_tab.get(&tab_id).copied().unwrap_or(0)
    }

    pub fn clear_tab(&mut self, tab_id: u64) {
        self.blocked_by_tab.remove(&tab_id);
    }

    pub fn enabled_for_url(&self, url: &str) -> bool {
        site_host(url)
            .map(|host| !self.disabled_sites.contains(&host))
            .unwrap_or(true)
    }

    pub fn set_enabled_for_url(&mut self, url: &str, enabled: bool) -> Option<String> {
        let host = site_host(url)?;
        if enabled {
            self.disabled_sites.remove(&host);
        } else {
            self.disabled_sites.insert(host.clone());
        }
        Some(host)
    }

    pub fn disabled_sites(&self) -> HashSet<String> {
        self.disabled_sites.clone()
    }
}

fn site_host(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()?
        .host_str()
        .map(|host| host.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_rules_block_known_tracker() {
        let temp = std::env::temp_dir().join("clearlane-shields-empty");
        let mut shields = Shields::load(&temp, HashSet::new());
        assert!(shields.should_block(
            1,
            "https://example.com/",
            "https://stats.doubleclick.net/pixel"
        ));
        assert_eq!(shields.blocked_count(1), 1);
    }

    #[test]
    fn per_site_override_disables_blocking() {
        let temp = std::env::temp_dir().join("clearlane-shields-empty-override");
        let mut shields = Shields::load(&temp, HashSet::new());
        shields.set_enabled_for_url("https://example.com", false);
        assert!(!shields.should_block(
            1,
            "https://example.com/",
            "https://stats.doubleclick.net/pixel"
        ));
    }
}
