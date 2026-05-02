use rusqlite::{params, Connection};
use crate::util::error::Result;

use super::models::{Movie, WatchlistItem};

const WORKFLOW_BLOCK_START: &str = "[AI-Movie-Player Workflow Studio]";
const WORKFLOW_BLOCK_END: &str = "[/AI-Movie-Player Workflow Studio]";

pub fn add_to_watchlist(conn: &Connection, item: &WatchlistItem) -> Result<i64> {
    conn.execute(
        "INSERT INTO watchlist (movie_id, tmdb_id, status, user_rating, notes, watched_date)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![item.movie_id, item.tmdb_id, item.status, item.user_rating, item.notes, item.watched_date],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_watchlist(conn: &Connection, item: &WatchlistItem) -> Result<()> {
    conn.execute(
        "UPDATE watchlist SET status=?1, user_rating=?2, notes=?3, watched_date=?4 WHERE id=?5",
        params![item.status, item.user_rating, item.notes, item.watched_date, item.id],
    )?;
    Ok(())
}

pub fn get_watchlist(conn: &Connection, status_filter: Option<&str>) -> Result<Vec<WatchlistItem>> {
    let sql = match status_filter {
        Some(_) => "SELECT * FROM watchlist WHERE status = ?1 ORDER BY added_date DESC",
        None => "SELECT * FROM watchlist ORDER BY added_date DESC",
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = match status_filter {
        Some(s) => stmt.query_map(params![s], watchlist_from_row)?,
        None => stmt.query_map([], watchlist_from_row)?,
    };
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_watchlist_item_for_movie(conn: &Connection, movie_id: i64) -> Result<Option<WatchlistItem>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM watchlist WHERE movie_id = ?1 ORDER BY added_date DESC LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![movie_id], watchlist_from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn upsert_workflow_summary(
    conn: &Connection,
    movie: &Movie,
    default_status: &str,
    workflow_text: &str,
) -> Result<()> {
    let existing = get_watchlist_item_for_movie(conn, movie.id)?;
    let merged_notes = merge_workflow_summary(
        existing.as_ref().and_then(|item| item.notes.as_deref()),
        workflow_text,
    );

    if let Some(mut item) = existing {
        item.notes = Some(merged_notes);
        update_watchlist(conn, &item)?;
    } else {
        let item = WatchlistItem {
            id: 0,
            movie_id: Some(movie.id),
            tmdb_id: movie.tmdb_id,
            status: default_status.to_string(),
            user_rating: None,
            notes: Some(merged_notes),
            added_date: String::new(),
            watched_date: None,
        };
        add_to_watchlist(conn, &item)?;
    }

    Ok(())
}

pub fn extract_workflow_summary(notes: &str) -> Option<String> {
    let start = notes.find(WORKFLOW_BLOCK_START)?;
    let end = notes.find(WORKFLOW_BLOCK_END)?;
    if end <= start {
        return None;
    }

    let body = &notes[start + WORKFLOW_BLOCK_START.len()..end];
    let trimmed = body.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn remove_from_watchlist(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM watchlist WHERE id = ?1", params![id])?;
    Ok(())
}

fn merge_workflow_summary(existing_notes: Option<&str>, workflow_text: &str) -> String {
    let block = format!(
        "{}\n{}\n{}",
        WORKFLOW_BLOCK_START,
        workflow_text.trim(),
        WORKFLOW_BLOCK_END
    );

    match existing_notes {
        Some(existing) if existing.contains(WORKFLOW_BLOCK_START) && existing.contains(WORKFLOW_BLOCK_END) => {
            let start = existing.find(WORKFLOW_BLOCK_START).unwrap_or(0);
            let end = existing.find(WORKFLOW_BLOCK_END).unwrap_or(existing.len()) + WORKFLOW_BLOCK_END.len();
            let prefix = existing[..start].trim_end();
            let suffix = existing[end..].trim_start();

            match (prefix.is_empty(), suffix.is_empty()) {
                (true, true) => block,
                (false, true) => format!("{}\n\n{}", prefix, block),
                (true, false) => format!("{}\n\n{}", block, suffix),
                (false, false) => format!("{}\n\n{}\n\n{}", prefix, block, suffix),
            }
        }
        Some(existing) if existing.trim().is_empty() => block,
        Some(existing) => format!("{}\n\n{}", existing.trim_end(), block),
        None => block,
    }
}

fn watchlist_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WatchlistItem> {
    Ok(WatchlistItem {
        id: row.get(0)?, movie_id: row.get(1)?, tmdb_id: row.get(2)?,
        status: row.get(3)?, user_rating: row.get(4)?, notes: row.get(5)?,
        added_date: row.get(6)?, watched_date: row.get(7)?,
    })
}
