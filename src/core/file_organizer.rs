//! File organization and renaming using configurable templates.

use std::path::{Path, PathBuf};

use crate::db::models::Movie;

/// A pending file rename operation.
#[derive(Debug, Clone)]
pub struct RenameOp {
    pub movie_id: i64,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
}

pub fn render_template(template: &str, movie: &Movie, parsed_title: Option<&str>) -> String {
    let mut result = template.to_string();
    let title = parsed_title.unwrap_or(&movie.title);

    result = result.replace("{title}", title);
    if let Some(ref cn) = movie.title_cn {
        result = result.replace("{title_cn}", cn);
    }
    if let Some(y) = movie.year {
        result = result.replace("{year}", &y.to_string());
    }
    if let Some(ref r) = movie.resolution {
        result = result.replace("{resolution}", r);
    }
    if let Some(ref s) = movie.source {
        result = result.replace("{source}", s);
    }
    if let Some(ref c) = movie.codec {
        result = result.replace("{codec}", c);
    }
    result
}

pub fn preview_rename(movie: &Movie, template: &str) -> Option<RenameOp> {
    let file_path = movie.local_file_path.as_ref()?;
    let old_path = PathBuf::from(file_path);

    let ext = old_path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();

    let new_name = render_template(template, movie, None) + &ext;

    let new_name = sanitize_filename(&new_name);

    let parent = old_path.parent()?;
    let new_path = parent.join(&new_name);

    if old_path == new_path {
        return None; // No change needed
    }

    Some(RenameOp { movie_id: movie.id, old_path, new_path })
}

fn sanitize_filename(name: &str) -> String {
    let mut sanitized = String::with_capacity(name.len());

    for ch in name.chars() {
        match ch {
            '/' | '\\' | '*' | '?' | '<' | '>' | '|' => {}
            ':' => sanitized.push_str(" -"),
            '"' => sanitized.push('\''),
            _ => sanitized.push(ch),
        }
    }

    sanitized
}

pub fn execute_rename(op: &RenameOp) -> std::io::Result<()> {
    if op.new_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Target already exists: {}", op.new_path.display()),
        ));
    }
    std::fs::rename(&op.old_path, &op.new_path)
}

pub fn batch_preview(movies: &[Movie], template: &str) -> Vec<RenameOp> {
    movies.iter().filter_map(|m| preview_rename(m, template)).collect()
}

pub fn safe_move_file(src: &Path, dst_dir: &Path) -> std::io::Result<PathBuf> {
    let filename = src.file_name().unwrap_or_default();
    let dst = dst_dir.join(filename);
    std::fs::create_dir_all(dst_dir)?;
    std::fs::rename(src, &dst)?;
    Ok(dst)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn sample_movie() -> Movie {
        Movie {
            id: 42,
            tmdb_id: Some(1042),
            imdb_id: Some("tt0000042".to_string()),
            title: "Hero: Rise/ Fall?".to_string(),
            title_cn: Some("英雄崛起".to_string()),
            original_title: None,
            year: Some(2024),
            release_date: None,
            poster_path: None,
            poster_local: None,
            backdrop_path: None,
            backdrop_local: None,
            rating: None,
            rating_count: None,
            genres: None,
            runtime: None,
            overview: None,
            overview_cn: None,
            tagline: None,
            director: None,
            cast_list: None,
            language: None,
            country: None,
            local_file_path: Some("C:/movies/original.mkv".to_string()),
            file_size: None,
            file_hash: None,
            resolution: Some("2160p".to_string()),
            source: Some("BluRay".to_string()),
            codec: Some("x265".to_string()),
            audio_langs: None,
            added_date: "2026-05-03T00:00:00Z".to_string(),
            updated_date: "2026-05-03T00:00:00Z".to_string(),
            tmdb_data: None,
        }
    }

    #[test]
    fn render_template_uses_movie_fields_and_parsed_title_override() {
        let movie = sample_movie();

        let rendered = render_template(
            "{title} ({year}) [{resolution}] {source} {codec}",
            &movie,
            Some("Hero Rise Fall"),
        );

        assert_eq!(rendered, "Hero Rise Fall (2024) [2160p] BluRay x265");
    }

    #[test]
    fn preview_rename_sanitizes_windows_invalid_characters() {
        let movie = sample_movie();

        let op = preview_rename(&movie, "{title} ({year})").expect("rename preview should exist");

        assert_eq!(op.movie_id, 42);
        assert_eq!(op.old_path, PathBuf::from("C:/movies/original.mkv"));
        assert_eq!(op.new_path, PathBuf::from("C:/movies/Hero - Rise Fall (2024).mkv"));
    }

    #[test]
    fn preview_rename_returns_none_when_name_does_not_change() {
        let mut movie = sample_movie();
        movie.title = "original".to_string();

        let op = preview_rename(&movie, "{title}");
        assert!(op.is_none());
    }

    #[test]
    fn batch_preview_skips_movies_without_rename_changes() {
        let mut unchanged = sample_movie();
        unchanged.id = 1;
        unchanged.title = "original".to_string();
        unchanged.local_file_path = Some("C:/movies/original (2024).mkv".to_string());

        let mut changed = sample_movie();
        changed.id = 2;
        changed.local_file_path = Some("C:/movies/second.mkv".to_string());

        let previews = batch_preview(&[unchanged, changed], "{title} ({year})");

        assert_eq!(previews.len(), 1);
        assert_eq!(previews[0].movie_id, 2);
    }

    #[test]
    fn safe_move_file_moves_into_destination_directory() {
        let temp = TempDir::new().expect("temp dir");
        let src_dir = temp.path().join("src");
        let dst_dir = temp.path().join("dst");
        std::fs::create_dir_all(&src_dir).expect("create src dir");

        let src_file = src_dir.join("movie.mkv");
        std::fs::write(&src_file, b"movie-data").expect("write source file");

        let moved_path = safe_move_file(&src_file, &dst_dir).expect("move should succeed");

        assert_eq!(moved_path, dst_dir.join("movie.mkv"));
        assert!(moved_path.exists());
        assert!(!src_file.exists());
    }
}
