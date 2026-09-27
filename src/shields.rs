use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use adblock::{
    Engine,
    lists::{FilterSet, ParseOptions},
    request::Request,
    resources::{PermissionMask, Resource},
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

const FILTER_FILES: &[&str] = &[
    "easylist.txt",
    "easyprivacy.txt",
    "ublock-filters.txt",
    "ublock-privacy.txt",
    "ublock-quick-fixes.txt",
];

const UBO_PERMISSION: PermissionMask = PermissionMask::from_bits(0b0000_0001);

pub struct Shields {
    engine: Engine,
    disabled_sites: HashSet<String>,
    blocked_by_tab: HashMap<u64, u64>,
}

impl Shields {
    pub fn fallback(disabled_sites: HashSet<String>) -> Self {
        Self {
            engine: build_engine(None),
            disabled_sites,
            blocked_by_tab: HashMap::new(),
        }
    }

    pub fn load(filter_dir: &Path, disabled_sites: HashSet<String>) -> Self {
        Self {
            engine: build_engine(Some(filter_dir)),
            disabled_sites,
            blocked_by_tab: HashMap::new(),
        }
    }

    pub fn build_full_engine(filter_dir: &Path) -> Engine {
        build_engine(Some(filter_dir))
    }

    pub fn replace_engine(&mut self, engine: Engine) {
        self.engine = engine;
    }

    pub fn should_block(
        &mut self,
        tab_id: u64,
        page_url: &str,
        request_url: &str,
        resource_type: &str,
        method: &str,
    ) -> bool {
        if !self.enabled_for_url(page_url) {
            return false;
        }
        let Ok(request) = Request::new(request_url, page_url, resource_type, method) else {
            return false;
        };
        let blocked = self.engine.check_network_request(&request).should_block();
        if blocked {
            *self.blocked_by_tab.entry(tab_id).or_default() += 1;
        }
        blocked
    }

    /// Returns page-start JavaScript containing hostname-specific cosmetic CSS and scriptlets.
    /// Generic class/id rules are intentionally left out for now; those need a MutationObserver
    /// bridge so only selectors that can actually match the live page are requested from adblock.
    pub fn cosmetic_script(&self, page_url: &str) -> Option<String> {
        if !self.enabled_for_url(page_url) {
            return None;
        }

        let resources = self.engine.url_cosmetic_resources(page_url);
        if resources.hide_selectors.is_empty() && resources.injected_script.trim().is_empty() {
            return None;
        }

        let mut selectors: Vec<_> = resources.hide_selectors.into_iter().collect();
        selectors.sort_unstable();
        let css = if selectors.is_empty() {
            String::new()
        } else {
            format!("{}{{display:none!important}}", selectors.join(","))
        };
        let css_json = serde_json::to_string(&css).ok()?;
        let scriptlets = resources.injected_script;

        Some(format!(
            r#"(()=>{{
const css={css_json};
if(css){{
  const install=()=>{{
    if(document.querySelector('style[data-clearlane-shields]')) return;
    const style=document.createElement('style');
    style.setAttribute('data-clearlane-shields','');
    style.textContent=css;
    (document.documentElement||document.head).appendChild(style);
  }};
  if(document.documentElement) install();
  else document.addEventListener('DOMContentLoaded',install,{{once:true}});
}}
{scriptlets}
}})();"#
        ))
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

fn build_engine(filter_dir: Option<&Path>) -> Engine {
    let mut set = FilterSet::new(false);
    set.add_filter_list(FALLBACK_RULES.to_string(), ParseOptions::default());
    if let Some(filter_dir) = filter_dir {
        for name in FILTER_FILES {
            if let Ok(text) = fs::read_to_string(filter_dir.join(name)) {
                let permissions = if name.starts_with("ublock-") {
                    UBO_PERMISSION
                } else {
                    PermissionMask::default()
                };
                set.add_filter_list(
                    text,
                    ParseOptions {
                        permissions,
                        ..ParseOptions::default()
                    },
                );
            }
        }
    }

    let mut engine = Engine::new_with_filter_set(set);
    if let Some(filter_dir) = filter_dir
        && let Ok(text) = fs::read_to_string(filter_dir.join("resources.json"))
        && let Ok(resources) = serde_json::from_str::<Vec<Resource>>(&text)
    {
        engine.use_resources(resources);
    }
    engine
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
    use adblock::resources::{MimeType, ResourceType};
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    fn test_dir(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "clearlane-shields-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create Shields test directory");
        path
    }

    #[test]
    fn fallback_rules_block_known_tracker() {
        let temp = std::env::temp_dir().join("clearlane-shields-empty");
        let mut shields = Shields::load(&temp, HashSet::new());
        assert!(shields.should_block(
            1,
            "https://example.com/",
            "https://stats.doubleclick.net/pixel",
            "image",
            "get"
        ));
        assert_eq!(shields.blocked_count(1), 1);
    }

    #[test]
    fn resource_type_specific_rules_are_respected() {
        let temp = test_dir("typed-network");
        fs::write(temp.join("easylist.txt"), "||ads.example^$script\n")
            .expect("write test filter list");
        let mut shields = Shields::load(&temp, HashSet::new());

        assert!(shields.should_block(
            1,
            "https://site.example/",
            "https://ads.example/ad.js",
            "script",
            "get"
        ));
        assert!(!shields.should_block(
            1,
            "https://site.example/",
            "https://ads.example/banner.png",
            "image",
            "get"
        ));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn official_ublock_scriptlets_are_authorized_and_emitted() {
        let temp = test_dir("scriptlet");
        fs::write(
            temp.join("ublock-filters.txt"),
            "example.com##+js(clearlane-test)\n",
        )
        .expect("write scriptlet filter");

        let resources = vec![Resource {
            name: "clearlane-test.js".into(),
            aliases: vec!["clearlane-test".into()],
            kind: ResourceType::Mime(MimeType::ApplicationJavascript),
            content: STANDARD.encode("window.__clearlane_scriptlet_test = 1;"),
            dependencies: vec![],
            permission: UBO_PERMISSION,
        }];
        fs::write(
            temp.join("resources.json"),
            serde_json::to_string(&resources).expect("serialize test resources"),
        )
        .expect("write scriptlet resources");

        let shields = Shields::load(&temp, HashSet::new());
        let script = shields
            .cosmetic_script("https://example.com/")
            .expect("scriptlet should be emitted");
        assert!(script.contains("__clearlane_scriptlet_test"));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn per_site_override_disables_blocking() {
        let temp = std::env::temp_dir().join("clearlane-shields-empty-override");
        let mut shields = Shields::load(&temp, HashSet::new());
        shields.set_enabled_for_url("https://example.com", false);
        assert!(!shields.should_block(
            1,
            "https://example.com/",
            "https://stats.doubleclick.net/pixel",
            "image",
            "get"
        ));
    }

    #[test]
    fn fallback_has_no_cosmetic_script_without_cosmetic_rules() {
        let shields = Shields::fallback(HashSet::new());
        assert!(shields.cosmetic_script("https://example.com/").is_none());
    }
}
