use std::path::{Path, PathBuf};

use crate::db::models::Movie;

#[derive(Debug, Clone)]
pub struct RenameOp {
    pub movie_id: i64,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
}

pub fn render_template(template: &str, movie: &Movie, parsed_title: Option<&str>) -> String {
    let mut result = template.to_string();

    result = result.replace("{title}", &movie.title);
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
    if let Some(t) = parsed_title {
        result = result.replace("{title}", t);
    }

    result
}

pub fn preview_rename(movie: &Movie, template: &str) -> Option<RenameOp> {
    let file_path = movie.local_file_path.as_ref()?;
    let old_path = PathBuf::from(file_path);

    let ext = old_path.extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let new_name = render_template(template, movie, None) + &ext;

    // Sanitize filename: remove characters invalid on Windows
    let new_name = new_name
        .replace('/', "")
        .replace('\\', "")
        .replace(':', " -")
        .replace('*', "")
        .replace('?', "")
        .replace('"', "'")
        .replace('<', "")
        .replace('>', "")
        .replace('|', "");

    let parent = old_path.parent()?;
    let new_path = parent.join(&new_name);

    if old_path == new_path {
        return None; // No change needed
    }

    Some(RenameOp {
        movie_id: movie.id,
        old_path,
        new_path,
    })
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
    movies
        .iter()
        .filter_map(|m| preview_rename(m, template))
        .collect()
}

pub fn safe_move_file(src: &Path, dst_dir: &Path) -> std::io::Result<PathBuf> {
    let filename = src.file_name().unwrap_or_default();
    let dst = dst_dir.join(filename);
    std::fs::create_dir_all(dst_dir)?;
    std::fs::rename(src, &dst)?;
    Ok(dst)
}
