//! Subtitle metadata storage and retrieval.

use rusqlite::{params, Connection};
use crate::util::error::Result;

use super::models::Subtitle;

/// Insert a subtitle record and return its ID.
pub fn insert_subtitle(conn: &Connection, s: &Subtitle) -> Result<i64> {
    conn.execute(
        "INSERT INTO subtitles (movie_id, language, language_label, source, source_url, file_name,
         local_path, file_size, rating, download_count, is_ai, is_hearing_imp, format, encoding, sync_status)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        params![
            s.movie_id, &s.language, s.language_label, &s.source, s.source_url, s.file_name,
            s.local_path, s.file_size, s.rating, s.download_count, s.is_ai as i32, s.is_hearing_imp as i32,
            s.format, s.encoding, s.sync_status,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_subtitles_for_movie(conn: &Connection, movie_id: i64) -> Result<Vec<Subtitle>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM subtitles WHERE movie_id = ?1 ORDER BY download_date DESC"
    )?;
    let rows = stmt.query_map(params![movie_id], subtitle_from_row)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_subtitle_count_for_movie(conn: &Connection, movie_id: i64) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM subtitles WHERE movie_id = ?1",
        params![movie_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn delete_subtitle(conn: &Connection, id: i64) -> Result<()> {
    if let Some(sub) = get_subtitle_by_id(conn, id)? {
        if let Some(path) = sub.local_path {
            std::fs::remove_file(&path).ok();
        }
    }
    conn.execute("DELETE FROM subtitles WHERE id = ?1", params![id])?;
    Ok(())
}

fn get_subtitle_by_id(conn: &Connection, id: i64) -> Result<Option<Subtitle>> {
    let mut stmt = conn.prepare("SELECT * FROM subtitles WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![id], subtitle_from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

fn subtitle_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Subtitle> {
    Ok(Subtitle {
        id: row.get(0)?, movie_id: row.get(1)?, language: row.get(2)?,
        language_label: row.get(3)?, source: row.get(4)?, source_url: row.get(5)?,
        file_name: row.get(6)?, local_path: row.get(7)?, file_size: row.get(8)?,
        rating: row.get(9)?, download_count: row.get(10)?,
        is_ai: row.get::<_, i32>(11)? != 0, is_hearing_imp: row.get::<_, i32>(12)? != 0,
        format: row.get(13)?, encoding: row.get(14)?, sync_status: row.get(15)?,
        download_date: row.get(16)?,
    })
}
