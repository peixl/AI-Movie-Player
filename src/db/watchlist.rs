//! Watchlist management with workflow card storage in notes field.

use crate::util::error::Result;
use rusqlite::{Connection, params};

use super::models::{Movie, WatchlistItem};

const WORKFLOW_BLOCK_START: &str = "[AI-Movie-Player Workflow Studio]";
const WORKFLOW_BLOCK_END: &str = "[/AI-Movie-Player Workflow Studio]";

/// Add a movie to the watchlist and return the new entry ID.
pub fn add_to_watchlist(conn: &Connection, item: &WatchlistItem) -> Result<i64> {
    conn.execute(
        "INSERT INTO watchlist (movie_id, tmdb_id, status, user_rating, notes, watched_date)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            item.movie_id,
            item.tmdb_id,
            item.status,
            item.user_rating,
            item.notes,
            item.watched_date
        ],
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

pub fn get_watchlist_item_for_movie(
    conn: &Connection,
    movie_id: i64,
) -> Result<Option<WatchlistItem>> {
    let mut stmt = conn
        .prepare("SELECT * FROM watchlist WHERE movie_id = ?1 ORDER BY added_date DESC LIMIT 1")?;
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
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

pub fn remove_from_watchlist(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM watchlist WHERE id = ?1", params![id])?;
    Ok(())
}

fn merge_workflow_summary(existing_notes: Option<&str>, workflow_text: &str) -> String {
    let block =
        format!("{}\n{}\n{}", WORKFLOW_BLOCK_START, workflow_text.trim(), WORKFLOW_BLOCK_END);

    match existing_notes {
        Some(existing)
            if existing.contains(WORKFLOW_BLOCK_START) && existing.contains(WORKFLOW_BLOCK_END) =>
        {
            let start = existing.find(WORKFLOW_BLOCK_START).unwrap_or(0);
            let end = existing.find(WORKFLOW_BLOCK_END).unwrap_or(existing.len())
                + WORKFLOW_BLOCK_END.len();
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
        id: row.get(0)?,
        movie_id: row.get(1)?,
        tmdb_id: row.get(2)?,
        status: row.get(3)?,
        user_rating: row.get(4)?,
        notes: row.get(5)?,
        added_date: row.get(6)?,
        watched_date: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        WORKFLOW_BLOCK_END, WORKFLOW_BLOCK_START, extract_workflow_summary, merge_workflow_summary,
    };

    #[test]
    fn merge_workflow_summary_appends_block_to_plain_notes() {
        let merged = merge_workflow_summary(Some("Personal note"), "Workflow title\nSummary line");

        assert!(merged.starts_with("Personal note\n\n"));
        assert!(merged.contains(WORKFLOW_BLOCK_START));
        assert!(merged.contains("Workflow title\nSummary line"));
        assert!(merged.ends_with(WORKFLOW_BLOCK_END));
    }

    #[test]
    fn merge_workflow_summary_replaces_existing_block_and_preserves_surrounding_notes() {
        let existing = format!(
            "Before note\n\n{}\nOld workflow\n{}\n\nAfter note",
            WORKFLOW_BLOCK_START, WORKFLOW_BLOCK_END
        );

        let merged = merge_workflow_summary(Some(&existing), "New workflow\nFresh summary");

        assert!(merged.contains("Before note"));
        assert!(merged.contains("After note"));
        assert!(merged.contains("New workflow\nFresh summary"));
        assert!(!merged.contains("Old workflow"));
        assert_eq!(merged.matches(WORKFLOW_BLOCK_START).count(), 1);
        assert_eq!(merged.matches(WORKFLOW_BLOCK_END).count(), 1);
    }

    #[test]
    fn merge_workflow_summary_normalizes_empty_existing_notes() {
        let merged = merge_workflow_summary(Some("   \n\t  "), "Workflow body");

        assert_eq!(
            merged,
            format!("{}\nWorkflow body\n{}", WORKFLOW_BLOCK_START, WORKFLOW_BLOCK_END)
        );
    }

    #[test]
    fn extract_workflow_summary_returns_trimmed_body() {
        let notes = format!(
            "Before\n{}\n  Workflow title\nSummary line  \n{}\nAfter",
            WORKFLOW_BLOCK_START, WORKFLOW_BLOCK_END
        );

        let extracted = extract_workflow_summary(&notes);

        assert_eq!(extracted.as_deref(), Some("Workflow title\nSummary line"));
    }

    #[test]
    fn extract_workflow_summary_rejects_empty_or_invalid_blocks() {
        let empty_notes = format!("{}\n   \n{}", WORKFLOW_BLOCK_START, WORKFLOW_BLOCK_END);
        let invalid_notes = format!("{} stray {}", WORKFLOW_BLOCK_END, WORKFLOW_BLOCK_START);

        assert_eq!(extract_workflow_summary(&empty_notes), None);
        assert_eq!(extract_workflow_summary(&invalid_notes), None);
    }
}
