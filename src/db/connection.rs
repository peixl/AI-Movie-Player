//! Database connection setup with WAL mode and automatic migrations.

use rusqlite::Connection;
use std::path::PathBuf;

use super::migrations;

/// Open (or create) the SQLite database with WAL mode and foreign keys enabled.
///
/// Handles legacy database path migration from `ai-movie-box.db` to `ai-movie-player.db`.
pub fn open_database(app_data_dir: &PathBuf) -> rusqlite::Result<Connection> {
    std::fs::create_dir_all(app_data_dir).ok();
    let preferred_path = app_data_dir.join("ai-movie-player.db");
    let legacy_path = app_data_dir.join("ai-movie-box.db");
    let db_path = if legacy_path.exists() { legacy_path } else { preferred_path };
    let conn = Connection::open(&db_path)?;

    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;

    run_migrations(&conn)?;

    Ok(conn)
}

fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    let current_version: i64 = conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM _migrations", [], |row| row.get(0))
        .unwrap_or(0);

    for (i, sql) in migrations::MIGRATIONS.iter().enumerate() {
        let version = i as i64 + 1;
        if version > current_version {
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO _migrations (id) VALUES (?1)", [version])?;
        }
    }

    Ok(())
}
