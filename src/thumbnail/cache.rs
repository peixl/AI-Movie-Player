//! Poster thumbnail download and disk caching.
//!
//! Image decoding/resizing runs on the blocking pool so a slow network volume
//! can never stall the UI thread. Files are written atomically via a temp
//! sibling path.

use std::path::{Path, PathBuf};

use crate::api::tmdb::TmdbClient;
use crate::util::error::Result;

const THUMBNAIL_WIDTH: u32 = 300;

/// Get the local path for a cached thumbnail by TMDB ID.
pub fn get_thumbnail_path(thumbnail_dir: &Path, tmdb_id: i64) -> PathBuf {
    thumbnail_dir.join(format!("{}_thumb.jpg", tmdb_id))
}

pub fn thumbnail_exists(thumbnail_dir: &Path, tmdb_id: i64) -> bool {
    get_thumbnail_path(thumbnail_dir, tmdb_id).exists()
}

pub async fn download_and_cache(
    client: &TmdbClient,
    poster_path: &str,
    tmdb_id: i64,
    thumbnail_dir: &Path,
) -> Result<String> {
    let thumb_path = get_thumbnail_path(thumbnail_dir, tmdb_id);

    if thumb_path.exists() {
        return Ok(thumb_path.to_string_lossy().to_string());
    }

    std::fs::create_dir_all(thumbnail_dir)?;

    let poster_bytes = client.download_poster(poster_path, "w342").await?;

    let thumb_path_blocking = thumb_path.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let img = image::load_from_memory(&poster_bytes)?;
        let thumb = img.thumbnail(THUMBNAIL_WIDTH, u32::MAX);

        // Write to a temp sibling then rename so a partial download can never
        // leave a corrupt thumbnail that other code trusts on disk.
        let tmp_path = thumb_path_blocking.with_extension("jpg.tmp");
        thumb.save(&tmp_path)?;
        std::fs::rename(&tmp_path, &thumb_path_blocking)?;
        Ok(())
    })
    .await
    .map_err(|err| {
        crate::util::error::AppError::Parse(format!("thumbnail task failed: {err}"))
    })??;

    Ok(thumb_path.to_string_lossy().to_string())
}
