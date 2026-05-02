//! Movie CRUD operations, search, and filtering.

use rusqlite::{params, Connection};
use crate::util::error::Result;

use super::models::*;

/// Insert a new movie record and return its ID.
pub fn insert_movie(conn: &Connection, m: &Movie) -> Result<i64> {
    conn.execute(
        "INSERT INTO movies (tmdb_id, imdb_id, title, title_cn, original_title, year, release_date,
         poster_path, poster_local, backdrop_path, backdrop_local, rating, rating_count, genres,
         runtime, overview, overview_cn, tagline, director, cast_list, language, country,
         local_file_path, file_size, file_hash, resolution, source, codec, audio_langs, tmdb_data)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30)",
        params![
            m.tmdb_id, m.imdb_id, &m.title, m.title_cn, m.original_title, m.year, m.release_date,
            m.poster_path, m.poster_local, m.backdrop_path, m.backdrop_local, m.rating, m.rating_count, m.genres,
            m.runtime, m.overview, m.overview_cn, m.tagline, m.director, m.cast_list, m.language, m.country,
            m.local_file_path, m.file_size, m.file_hash, m.resolution, m.source, m.codec, m.audio_langs, m.tmdb_data,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_movie(conn: &Connection, m: &Movie) -> Result<()> {
    conn.execute(
        "UPDATE movies SET tmdb_id=?1, imdb_id=?2, title=?3, title_cn=?4, original_title=?5, year=?6,
         release_date=?7, poster_path=?8, poster_local=?9, backdrop_path=?10, backdrop_local=?11,
         rating=?12, rating_count=?13, genres=?14, runtime=?15, overview=?16, overview_cn=?17,
         tagline=?18, director=?19, cast_list=?20, language=?21, country=?22,
         local_file_path=?23, file_size=?24, file_hash=?25, resolution=?26, source=?27, codec=?28,
         audio_langs=?29, updated_date=datetime('now'), tmdb_data=?30
         WHERE id=?31",
        params![
            m.tmdb_id, m.imdb_id, &m.title, m.title_cn, m.original_title, m.year, m.release_date,
            m.poster_path, m.poster_local, m.backdrop_path, m.backdrop_local, m.rating, m.rating_count, m.genres,
            m.runtime, m.overview, m.overview_cn, m.tagline, m.director, m.cast_list, m.language, m.country,
            m.local_file_path, m.file_size, m.file_hash, m.resolution, m.source, m.codec, m.audio_langs, m.tmdb_data,
            m.id,
        ],
    )?;
    Ok(())
}

pub fn get_movie_by_id(conn: &Connection, id: i64) -> Result<Option<Movie>> {
    let mut stmt = conn.prepare("SELECT * FROM movies WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![id], movie_from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn get_movie_by_tmdb_id(conn: &Connection, tmdb_id: i64) -> Result<Option<Movie>> {
    let mut stmt = conn.prepare("SELECT * FROM movies WHERE tmdb_id = ?1")?;
    let mut rows = stmt.query_map(params![tmdb_id], movie_from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn get_all_movie_summaries(
    conn: &Connection,
    sort_by: &str,
    ascending: bool,
    genre_filter: Option<&str>,
    search: Option<&str>,
) -> Result<Vec<MovieSummary>> {
    let mut sql = String::from(
        "SELECT id, title, title_cn, year, poster_local, poster_path, rating, genres, resolution, added_date FROM movies WHERE 1=1"
    );

    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(g) = genre_filter {
        sql.push_str(" AND genres LIKE ?");
        param_values.push(Box::new(format!("%{}%", g)));
    }
    if let Some(s) = search {
        // Try FTS5 first, fall back to LIKE
        let fts_results = conn
            .prepare(
                "SELECT rowid FROM movies_fts WHERE movies_fts MATCH ? ORDER BY rank LIMIT 1000",
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![s], |row| row.get::<_, i64>(0))?;
                let ids: Vec<i64> = rows.filter_map(|r| r.ok()).collect();
                Ok(ids)
            });

        if let Ok(ids) = fts_results {
            if !ids.is_empty() {
                let placeholders: Vec<String> =
                    ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
                sql.push_str(&format!(" AND id IN ({})", placeholders.join(",")));
                for id in ids {
                    param_values.push(Box::new(id));
                }
            } else {
                // No FTS matches — return empty set
                sql.push_str(" AND 1=0");
            }
        } else {
            // FTS not available, fall back to LIKE
            sql.push_str(" AND (title LIKE ? OR title_cn LIKE ? OR cast_list LIKE ?)");
            let pattern = format!("%{}%", s);
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern));
        }
    }

    let order = match sort_by {
        "title" => "title",
        "year" => "year",
        "rating" => "rating",
        _ => "added_date",
    };
    let dir = if ascending { "ASC" } else { "DESC" };
    sql.push_str(&format!(" ORDER BY {} {}", order, dir));

    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), summary_from_row)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_all_movies(conn: &Connection) -> Result<Vec<Movie>> {
    let mut stmt = conn.prepare("SELECT * FROM movies ORDER BY added_date DESC")?;
    let rows = stmt.query_map([], movie_from_row)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_movie_count(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM movies", [], |row| row.get(0))?;
    Ok(count)
}

pub fn delete_movie(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM movies WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn movie_exists_by_path(conn: &Connection, path: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM movies WHERE local_file_path = ?1",
        params![path],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn movie_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Movie> {
    Ok(Movie {
        id: row.get(0)?, tmdb_id: row.get(1)?, imdb_id: row.get(2)?, title: row.get(3)?,
        title_cn: row.get(4)?, original_title: row.get(5)?, year: row.get(6)?,
        release_date: row.get(7)?, poster_path: row.get(8)?, poster_local: row.get(9)?,
        backdrop_path: row.get(10)?, backdrop_local: row.get(11)?, rating: row.get(12)?,
        rating_count: row.get(13)?, genres: row.get(14)?, runtime: row.get(15)?,
        overview: row.get(16)?, overview_cn: row.get(17)?, tagline: row.get(18)?,
        director: row.get(19)?, cast_list: row.get(20)?, language: row.get(21)?,
        country: row.get(22)?, local_file_path: row.get(23)?, file_size: row.get(24)?,
        file_hash: row.get(25)?, resolution: row.get(26)?, source: row.get(27)?,
        codec: row.get(28)?, audio_langs: row.get(29)?, added_date: row.get(30)?,
        updated_date: row.get(31)?, tmdb_data: row.get(32)?,
    })
}

fn summary_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MovieSummary> {
    Ok(MovieSummary {
        id: row.get(0)?, title: row.get(1)?, title_cn: row.get(2)?, year: row.get(3)?,
        poster_local: row.get(4)?, poster_path: row.get(5)?, rating: row.get(6)?,
        genres: row.get(7)?, resolution: row.get(8)?, added_date: row.get(9)?,
    })
}
