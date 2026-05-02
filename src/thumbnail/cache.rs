//! Poster thumbnail download and disk caching.

use std::path::PathBuf;

use crate::api::tmdb::TmdbClient;
use crate::util::error::Result;

const THUMBNAIL_WIDTH: u32 = 300;

/// Get the local path for a cached thumbnail by TMDB ID.
pub fn get_thumbnail_path(thumbnail_dir: &PathBuf, tmdb_id: i64) -> PathBuf {
    thumbnail_dir.join(format!("{}_thumb.jpg", tmdb_id))
}

pub fn thumbnail_exists(thumbnail_dir: &PathBuf, tmdb_id: i64) -> bool {
    get_thumbnail_path(thumbnail_dir, tmdb_id).exists()
}

pub async fn download_and_cache(
    client: &TmdbClient,
    poster_path: &str,
    tmdb_id: i64,
    thumbnail_dir: &PathBuf,
) -> Result<String> {
    let thumb_path = get_thumbnail_path(thumbnail_dir, tmdb_id);

    if thumb_path.exists() {
        return Ok(thumb_path.to_string_lossy().to_string());
    }

    std::fs::create_dir_all(thumbnail_dir)?;

    let poster_bytes = client.download_poster(poster_path, "w342").await?;

    // Resize to thumbnail
    let img = image::load_from_memory(&poster_bytes)?;
    let thumb = img.thumbnail(THUMBNAIL_WIDTH, u32::MAX);
    thumb.save(&thumb_path)?;

    Ok(thumb_path.to_string_lossy().to_string())
}
