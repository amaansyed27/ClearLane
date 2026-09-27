use url::Url;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TabId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabState {
    pub id: TabId,
    pub url: String,
    pub title: String,
    pub loading: bool,
    pub shields_enabled: bool,
    pub blocked_count: u64,
    history: Vec<String>,
    history_index: usize,
}

impl TabState {
    fn new(id: TabId, url: String) -> Self {
        let title = title_for_url(&url);
        Self {
            id,
            history: vec![url.clone()],
            history_index: 0,
            title,
            url,
            loading: false,
            shields_enabled: true,
            blocked_count: 0,
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }
}

#[derive(Debug)]
pub struct BrowserState {
    tabs: Vec<TabState>,
    active: Option<TabId>,
    next_id: u64,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active: None,
            next_id: 1,
        }
    }
}

impl BrowserState {
    pub fn open_tab(&mut self, url: impl Into<String>) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        self.tabs.push(TabState::new(id, url.into()));
        self.active = Some(id);
        id
    }

    pub fn close_tab(&mut self, id: TabId) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return false;
        };

        self.tabs.remove(index);
        if self.active == Some(id) {
            self.active = self
                .tabs
                .get(index.min(self.tabs.len().saturating_sub(1)))
                .map(|tab| tab.id);
        }
        true
    }

    pub fn activate(&mut self, id: TabId) -> bool {
        if self.tabs.iter().any(|tab| tab.id == id) {
            self.active = Some(id);
            true
        } else {
            false
        }
    }

    pub fn active_id(&self) -> Option<TabId> {
        self.active
    }

    pub fn active(&self) -> Option<&TabState> {
        self.active.and_then(|id| self.tab(id))
    }

    pub fn tab(&self, id: TabId) -> Option<&TabState> {
        self.tabs.iter().find(|tab| tab.id == id)
    }

    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut TabState> {
        self.tabs.iter_mut().find(|tab| tab.id == id)
    }

    pub fn tabs(&self) -> &[TabState] {
        &self.tabs
    }

    pub fn navigate(&mut self, id: TabId, url: impl Into<String>) -> bool {
        let Some(tab) = self.tab_mut(id) else {
            return false;
        };
        let url = url.into();
        tab.history.truncate(tab.history_index + 1);
        tab.history.push(url.clone());
        tab.history_index = tab.history.len() - 1;
        tab.url = url;
        tab.title = title_for_url(&tab.url);
        tab.loading = false;
        true
    }

    pub fn go_back(&mut self, id: TabId) -> bool {
        let Some(tab) = self.tab_mut(id) else {
            return false;
        };
        if !tab.can_go_back() {
            return false;
        }
        tab.history_index -= 1;
        tab.url = tab.history[tab.history_index].clone();
        tab.title = title_for_url(&tab.url);
        tab.loading = false;
        true
    }

    pub fn go_forward(&mut self, id: TabId) -> bool {
        let Some(tab) = self.tab_mut(id) else {
            return false;
        };
        if !tab.can_go_forward() {
            return false;
        }
        tab.history_index += 1;
        tab.url = tab.history[tab.history_index].clone();
        tab.title = title_for_url(&tab.url);
        tab.loading = false;
        true
    }

    pub fn reload(&mut self, id: TabId) -> bool {
        let Some(tab) = self.tab_mut(id) else {
            return false;
        };
        tab.loading = true;
        true
    }

    pub fn stop(&mut self, id: TabId) -> bool {
        let Some(tab) = self.tab_mut(id) else {
            return false;
        };
        tab.loading = false;
        true
    }

    pub fn toggle_shields(&mut self, id: TabId) -> bool {
        let Some(tab) = self.tab_mut(id) else {
            return false;
        };
        tab.shields_enabled = !tab.shields_enabled;
        true
    }

    pub fn set_blocked_count(&mut self, id: TabId, count: u64) {
        if let Some(tab) = self.tab_mut(id) {
            tab.blocked_count = count;
        }
    }

    pub fn update_title(&mut self, id: TabId, title: impl Into<String>) {
        if let Some(tab) = self.tab_mut(id) {
            tab.title = title.into();
        }
    }
}

fn title_for_url(url: &str) -> String {
    if url == "clearlane://newtab" {
        return "New tab".into();
    }

    Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .map(|host| host.strip_prefix("www.").unwrap_or(&host).to_owned())
        .unwrap_or_else(|| "New tab".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_active_tab_selects_a_neighbor() {
        let mut browser = BrowserState::default();
        let first = browser.open_tab("https://one.example");
        let second = browser.open_tab("https://two.example");
        assert_eq!(browser.active_id(), Some(second));
        assert!(browser.close_tab(second));
        assert_eq!(browser.active_id(), Some(first));
    }

    #[test]
    fn navigation_history_drives_back_and_forward() {
        let mut browser = BrowserState::default();
        let tab = browser.open_tab("https://one.example");
        assert!(browser.navigate(tab, "https://two.example"));
        assert!(browser.tab(tab).is_some_and(TabState::can_go_back));
        assert!(browser.go_back(tab));
        assert_eq!(
            browser.tab(tab).map(|tab| tab.url.as_str()),
            Some("https://one.example")
        );
        assert!(browser.go_forward(tab));
        assert_eq!(
            browser.tab(tab).map(|tab| tab.url.as_str()),
            Some("https://two.example")
        );
    }

    #[test]
    fn reload_and_stop_are_explicit_fake_loading_states() {
        let mut browser = BrowserState::default();
        let tab = browser.open_tab("https://example.com");
        assert!(browser.reload(tab));
        assert!(browser.tab(tab).is_some_and(|tab| tab.loading));
        assert!(browser.stop(tab));
        assert!(browser.tab(tab).is_some_and(|tab| !tab.loading));
    }

    #[test]
    fn shields_toggle_is_per_tab() {
        let mut browser = BrowserState::default();
        let tab = browser.open_tab("https://example.com");
        assert!(browser.tab(tab).is_some_and(|tab| tab.shields_enabled));
        assert!(browser.toggle_shields(tab));
        assert!(browser.tab(tab).is_some_and(|tab| !tab.shields_enabled));
    }
}
