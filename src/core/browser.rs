#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TabId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabState {
    pub id: TabId,
    pub url: String,
    pub title: String,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

impl TabState {
    fn new(id: TabId, url: String) -> Self {
        Self {
            id,
            title: "New tab".to_string(),
            url,
            loading: true,
            can_go_back: false,
            can_go_forward: false,
        }
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
    pub fn open_tab(&mut self, url: String) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        self.tabs.push(TabState::new(id, url));
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

    pub fn update_address(&mut self, id: TabId, url: String) {
        if let Some(tab) = self.tab_mut(id) {
            tab.url = url;
        }
    }

    pub fn update_title(&mut self, id: TabId, title: String) {
        if let Some(tab) = self.tab_mut(id) {
            tab.title = if title.trim().is_empty() {
                tab.url.clone()
            } else {
                title
            };
        }
    }

    pub fn update_loading(&mut self, id: TabId, loading: bool, back: bool, forward: bool) {
        if let Some(tab) = self.tab_mut(id) {
            tab.loading = loading;
            tab.can_go_back = back;
            tab.can_go_forward = forward;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_active_tab_selects_a_neighbor() {
        let mut browser = BrowserState::default();
        let first = browser.open_tab("https://one.example".into());
        let second = browser.open_tab("https://two.example".into());
        assert_eq!(browser.active_id(), Some(second));
        assert!(browser.close_tab(second));
        assert_eq!(browser.active_id(), Some(first));
    }

    #[test]
    fn unknown_tab_operations_are_noops() {
        let mut browser = BrowserState::default();
        assert!(!browser.activate(TabId(99)));
        assert!(!browser.close_tab(TabId(99)));
    }
}
