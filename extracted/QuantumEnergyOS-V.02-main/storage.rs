// ═══════════════════════════════════════════════════════════════════════
//  storage.rs — SQLite offline-first (funciona sin internet)
// ═══════════════════════════════════════════════════════════════════════

use rusqlite::{Connection, Result as SqlResult, params};
use serde_json;
use std::path::PathBuf;
use crate::{GridBalanceResult, GridHistoryEntry};

pub struct QuantumStorage {
    db_path: PathBuf,
}

impl QuantumStorage {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        let storage = Self { db_path };
        storage.init_db().map_err(|e| e.to_string())?;
        Ok(storage)
    }

    fn connect(&self) -> SqlResult<Connection> {
        Connection::open(&self.db_path)
    }

    fn init_db(&self) -> SqlResult<()> {
        let conn = self.connect()?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS quartz4d_store (
                key       TEXT PRIMARY KEY,
                data      BLOB NOT NULL,
                layer     INTEGER DEFAULT 0,
                created   TEXT DEFAULT (datetime('now')),
                updated   TEXT DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS grid_history (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp     TEXT DEFAULT (datetime('now')),
                n_nodes       INTEGER,
                energy_saved  REAL,
                config        TEXT,
                quartz_hash   TEXT
            );
            CREATE TABLE IF NOT EXISTS solar_events (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp     TEXT DEFAULT (datetime('now')),
                kp_index      REAL,
                risk_level    TEXT,
                grid_impact   REAL
            );
            CREATE INDEX IF NOT EXISTS idx_grid_history_ts ON grid_history(timestamp);
        ")?;
        Ok(())
    }

    // ── Cuarzo 4D KV store ────────────────────────────────────────
    pub fn save(&self, key: &str, data: &[u8], layer: u8) -> SqlResult<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT OR REPLACE INTO quartz4d_store (key, data, layer, updated)
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![key, data, layer],
        )?;
        Ok(())
    }

    pub fn load(&self, key: &str) -> SqlResult<Vec<u8>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT data FROM quartz4d_store WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
    }

    // ── Historial de red ──────────────────────────────────────────
    pub fn save_grid_history(&self, result: &GridBalanceResult) -> SqlResult<()> {
        let conn = self.connect()?;
        let config = serde_json::to_string(&result.optimal_config)
            .unwrap_or_default();
        conn.execute(
            "INSERT INTO grid_history (n_nodes, energy_saved, config, quartz_hash)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                result.n_nodes,
                result.energy_saved_kw,
                config,
                result.quartz_hash,
            ],
        )?;
        Ok(())
    }

    pub fn get_grid_history(&self, limit: usize) -> SqlResult<Vec<GridHistoryEntry>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, n_nodes, energy_saved, config
             FROM grid_history ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(GridHistoryEntry {
                id:           row.get(0)?,
                timestamp:    row.get(1)?,
                n_nodes:      row.get::<_, i64>(2)? as usize,
                energy_saved: row.get(3)?,
                config:       row.get(4)?,
            })
        })?;
        rows.collect()
    }
}
