use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, params};

#[derive(Debug)]
pub(crate) struct PersistedState {
    pub urls: Vec<String>,
    pub active_index: usize,
    pub sidebar_open: bool,
    pub disabled_shields: HashSet<String>,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            urls: vec!["https://www.google.com/".to_string()],
            active_index: 0,
            sidebar_open: true,
            disabled_shields: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Persistence {
    path: PathBuf,
}

impl Persistence {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            path: state_dir.join("state.db"),
        }
    }

    fn connect(&self) -> rusqlite::Result<Connection> {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let connection = Connection::open(&self.path)?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS session_tabs (position INTEGER PRIMARY KEY, url TEXT NOT NULL);",
        )?;
        Ok(connection)
    }

    pub fn load(&self) -> PersistedState {
        self.load_inner().unwrap_or_default()
    }

    fn load_inner(&self) -> rusqlite::Result<PersistedState> {
        let connection = self.connect()?;
        let mut state = PersistedState::default();
        let mut statement = connection.prepare("SELECT url FROM session_tabs ORDER BY position")?;
        let urls = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if !urls.is_empty() {
            state.urls = urls;
        }
        state.active_index = read_setting(&connection, "active_index")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        state.sidebar_open = read_setting(&connection, "sidebar_open")
            .map(|value| value == "1")
            .unwrap_or(true);
        let mut shields =
            connection.prepare("SELECT key FROM settings WHERE key LIKE 'shield_off:%'")?;
        state.disabled_shields = shields
            .query_map([], |row| {
                let key: String = row.get(0)?;
                Ok(key.trim_start_matches("shield_off:").to_string())
            })?
            .collect::<rusqlite::Result<HashSet<_>>>()?;
        Ok(state)
    }

    pub fn save(
        &self,
        urls: &[String],
        active_index: usize,
        sidebar_open: bool,
        disabled: &HashSet<String>,
    ) {
        if let Err(error) = self.save_inner(urls, active_index, sidebar_open, disabled) {
            eprintln!("ClearLane persistence warning: {error}");
        }
    }

    fn save_inner(
        &self,
        urls: &[String],
        active_index: usize,
        sidebar_open: bool,
        disabled: &HashSet<String>,
    ) -> rusqlite::Result<()> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        transaction.execute("DELETE FROM session_tabs", [])?;
        for (position, url) in urls.iter().enumerate() {
            transaction.execute(
                "INSERT INTO session_tabs(position, url) VALUES (?1, ?2)",
                params![position as i64, url],
            )?;
        }
        transaction.execute(
            "INSERT OR REPLACE INTO settings(key,value) VALUES ('active_index',?1)",
            [active_index.to_string()],
        )?;
        transaction.execute(
            "INSERT OR REPLACE INTO settings(key,value) VALUES ('sidebar_open',?1)",
            [if sidebar_open { "1" } else { "0" }],
        )?;
        transaction.execute("DELETE FROM settings WHERE key LIKE 'shield_off:%'", [])?;
        for host in disabled {
            transaction.execute(
                "INSERT OR REPLACE INTO settings(key,value) VALUES (?1,'1')",
                [format!("shield_off:{host}")],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }
}

fn read_setting(connection: &Connection, key: &str) -> Option<String> {
    connection
        .query_row("SELECT value FROM settings WHERE key=?1", [key], |row| {
            row.get(0)
        })
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn round_trips_session_and_shield_overrides() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("clearlane-persistence-{unique}"));
        let persistence = Persistence::new(&dir);
        let urls = vec![
            "https://example.com/".to_string(),
            "https://rust-lang.org/".to_string(),
        ];
        let disabled = HashSet::from(["example.com".to_string()]);
        persistence.save(&urls, 1, false, &disabled);
        let loaded = persistence.load();
        assert_eq!(loaded.urls, urls);
        assert_eq!(loaded.active_index, 1);
        assert!(!loaded.sidebar_open);
        assert!(loaded.disabled_shields.contains("example.com"));
        let _ = fs::remove_dir_all(dir);
    }
}
