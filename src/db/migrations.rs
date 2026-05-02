//! Schema migrations for SQLite database versioning.
//!
//! Each migration is a SQL statement that creates or alters tables.
//! Migrations are tracked in the `_migrations` table and applied sequentially.

/// Ordered list of SQL migration statements.
pub const MIGRATIONS: &[&str] = &[
    // V1: Core tables
    r#"
CREATE TABLE movies (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tmdb_id         INTEGER UNIQUE,
    imdb_id         TEXT,
    title           TEXT NOT NULL,
    title_cn        TEXT,
    original_title  TEXT,
    year            INTEGER,
    release_date    TEXT,
    poster_path     TEXT,
    poster_local    TEXT,
    backdrop_path   TEXT,
    backdrop_local  TEXT,
    rating          REAL,
    rating_count    INTEGER,
    genres          TEXT,
    runtime         INTEGER,
    overview        TEXT,
    overview_cn     TEXT,
    tagline         TEXT,
    director        TEXT,
    cast_list       TEXT,
    language        TEXT,
    country         TEXT,
    local_file_path TEXT,
    file_size       INTEGER,
    file_hash       TEXT,
    resolution      TEXT,
    source          TEXT,
    codec           TEXT,
    audio_langs     TEXT,
    added_date      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_date    TEXT NOT NULL DEFAULT (datetime('now')),
    tmdb_data       TEXT
);

CREATE INDEX idx_movies_tmdb_id ON movies(tmdb_id);
CREATE INDEX idx_movies_title ON movies(title);
CREATE INDEX idx_movies_year ON movies(year);
CREATE INDEX idx_movies_rating ON movies(rating);
CREATE INDEX idx_movies_added_date ON movies(added_date);

CREATE TABLE subtitles (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    movie_id        INTEGER NOT NULL,
    language        TEXT NOT NULL,
    language_label  TEXT,
    source          TEXT NOT NULL,
    source_url      TEXT,
    file_name       TEXT,
    local_path      TEXT,
    file_size       INTEGER,
    rating          REAL,
    download_count  INTEGER,
    is_ai           INTEGER NOT NULL DEFAULT 0,
    is_hearing_imp  INTEGER NOT NULL DEFAULT 0,
    format          TEXT,
    encoding        TEXT,
    sync_status     TEXT DEFAULT 'unknown',
    download_date   TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (movie_id) REFERENCES movies(id) ON DELETE CASCADE
);

CREATE INDEX idx_subtitles_movie ON subtitles(movie_id);
CREATE INDEX idx_subtitles_lang ON subtitles(language);
CREATE INDEX idx_subtitles_source ON subtitles(source);

CREATE TABLE watchlist (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    movie_id        INTEGER,
    tmdb_id         INTEGER,
    status          TEXT NOT NULL DEFAULT 'want_to_watch',
    user_rating     REAL,
    notes           TEXT,
    added_date      TEXT NOT NULL DEFAULT (datetime('now')),
    watched_date    TEXT,
    FOREIGN KEY (movie_id) REFERENCES movies(id) ON DELETE SET NULL
);

CREATE INDEX idx_watchlist_status ON watchlist(status);

CREATE TABLE settings (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_date    TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO settings (key, value) VALUES
    ('tmdb_api_key', ''),
    ('tmdb_language', 'zh-CN'),
    ('subtitle_languages', '["zh","zh-CN","en"]'),
    ('rename_template', '{title} ({year})'),
    ('theme', 'dark');
"#,
    // V2: User tags
    r#"
CREATE TABLE user_tags (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    movie_id        INTEGER NOT NULL,
    tag             TEXT NOT NULL,
    FOREIGN KEY (movie_id) REFERENCES movies(id) ON DELETE CASCADE,
    UNIQUE(movie_id, tag)
);
"#,
    // V3: Full-text search
    r#"
CREATE VIRTUAL TABLE movies_fts USING fts5(
    title, title_cn, overview, overview_cn, director,
    content='movies',
    content_rowid='id'
);

CREATE TRIGGER movies_ai AFTER INSERT ON movies BEGIN
    INSERT INTO movies_fts(rowid, title, title_cn, overview, overview_cn, director)
    VALUES (new.id, new.title, new.title_cn, new.overview, new.overview_cn, new.director);
END;

CREATE TRIGGER movies_ad AFTER DELETE ON movies BEGIN
    INSERT INTO movies_fts(movies_fts, rowid, title, title_cn, overview, overview_cn, director)
    VALUES ('delete', old.id, old.title, old.title_cn, old.overview, old.overview_cn, old.director);
END;

CREATE TRIGGER movies_au AFTER UPDATE ON movies BEGIN
    INSERT INTO movies_fts(movies_fts, rowid, title, title_cn, overview, overview_cn, director)
    VALUES ('delete', old.id, old.title, old.title_cn, old.overview, old.overview_cn, old.director);
    INSERT INTO movies_fts(rowid, title, title_cn, overview, overview_cn, director)
    VALUES (new.id, new.title, new.title_cn, new.overview, new.overview_cn, new.director);
END;
"#,
];
