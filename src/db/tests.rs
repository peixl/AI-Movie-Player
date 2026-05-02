#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::TempDir;

    use crate::db::{connection, models::*, movies, settings, subtitles, watchlist};

    fn setup_db() -> (Connection, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let conn = connection::open_database(&dir.path().to_path_buf()).expect("Failed to open test DB");
        (conn, dir)
    }

    fn sample_movie(id: i64) -> Movie {
        Movie {
            id: 0, // Auto-assigned by SQLite
            tmdb_id: Some(100 + id),
            imdb_id: Some(format!("tt{:07}", id)),
            title: format!("Test Movie {}", id),
            title_cn: Some(format!("测试电影 {}", id)),
            original_title: None,
            year: Some(2020 + id as i32),
            release_date: Some(format!("{}-01-15", 2020 + id)),
            poster_path: Some(format!("/poster{}.jpg", id)),
            poster_local: Some(format!("/cache/poster{}.jpg", id)),
            backdrop_path: None,
            backdrop_local: None,
            rating: Some(7.5 + id as f64 * 0.1),
            rating_count: Some(1000 + id as i32 * 100),
            genres: Some(r#"["Action","Sci-Fi"]"#.to_string()),
            runtime: Some(120),
            overview: Some(format!("Overview of movie {}", id)),
            overview_cn: Some(format!("电影 {} 概述", id)),
            tagline: None,
            director: Some(format!("Director {}", id)),
            cast_list: Some(r#"[{"name":"Actor A","character":"Hero"}]"#.to_string()),
            language: Some("en".to_string()),
            country: Some("US".to_string()),
            local_file_path: Some(format!("/movies/movie{}.mkv", id)),
            file_size: Some(8_000_000_000),
            file_hash: Some(format!("{:064x}", id)),
            resolution: Some("1080p".to_string()),
            source: Some("BluRay".to_string()),
            codec: Some("x265".to_string()),
            audio_langs: Some(r#"["eng","spa"]"#.to_string()),
            added_date: "2024-01-15T00:00:00Z".to_string(),
            updated_date: "2024-01-15T00:00:00Z".to_string(),
            tmdb_data: None,
        }
    }

    fn sample_subtitle(movie_id: i64) -> Subtitle {
        Subtitle {
            id: 0,
            movie_id,
            language: "zh-CN".to_string(),
            language_label: Some("Chinese (Simplified)".to_string()),
            source: "opensubtitles".to_string(),
            source_url: Some("https://example.com/sub".to_string()),
            file_name: Some("movie.zh.srt".to_string()),
            local_path: None,
            file_size: None,
            rating: Some(9.0),
            download_count: Some(500),
            is_ai: false,
            is_hearing_imp: false,
            format: Some("srt".to_string()),
            encoding: Some("utf-8".to_string()),
            sync_status: Some("good".to_string()),
            download_date: "2024-01-15T00:00:00Z".to_string(),
        }
    }

    // --- Movie CRUD ---

    #[test]
    fn test_insert_and_get_movie() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        let id = movies::insert_movie(&conn, &movie).expect("Insert failed");
        assert!(id > 0);

        let fetched = movies::get_movie_by_id(&conn, id)
            .expect("Query failed")
            .expect("Movie not found");

        assert_eq!(fetched.title, "Test Movie 1");
        assert_eq!(fetched.year, Some(2021));
        assert_eq!(fetched.resolution.as_deref(), Some("1080p"));
    }

    #[test]
    fn test_update_movie() {
        let (conn, _dir) = setup_db();
        let mut movie = sample_movie(1);
        let id = movies::insert_movie(&conn, &movie).expect("Insert failed");
        movie.id = id;

        movie.title = "Updated Title".to_string();
        movies::update_movie(&conn, &movie).expect("Update failed");

        let fetched = movies::get_movie_by_id(&conn, id)
            .expect("Query failed")
            .expect("Movie not found");
        assert_eq!(fetched.title, "Updated Title");
    }

    #[test]
    fn test_get_movie_by_tmdb_id() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        movies::insert_movie(&conn, &movie).expect("Insert failed");

        let fetched = movies::get_movie_by_tmdb_id(&conn, 101)
            .expect("Query failed")
            .expect("Movie not found");
        assert_eq!(fetched.tmdb_id, Some(101));
    }

    #[test]
    fn test_movie_not_found() {
        let (conn, _dir) = setup_db();
        let result = movies::get_movie_by_id(&conn, 99999).expect("Query failed");
        assert!(result.is_none());
    }

    #[test]
    fn test_movie_exists_by_path() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        movies::insert_movie(&conn, &movie).expect("Insert failed");

        assert!(movies::movie_exists_by_path(&conn, "/movies/movie1.mkv").unwrap_or(false));
        assert!(!movies::movie_exists_by_path(&conn, "/nonexistent.mkv").unwrap_or(false));
    }

    #[test]
    fn test_delete_movie() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        let id = movies::insert_movie(&conn, &movie).expect("Insert failed");

        movies::delete_movie(&conn, id).expect("Delete failed");
        let result = movies::get_movie_by_id(&conn, id).expect("Query failed");
        assert!(result.is_none());
    }

    #[test]
    fn test_movie_count() {
        let (conn, _dir) = setup_db();
        assert_eq!(movies::get_movie_count(&conn).unwrap(), 0);

        movies::insert_movie(&conn, &sample_movie(1)).unwrap();
        movies::insert_movie(&conn, &sample_movie(2)).unwrap();

        assert_eq!(movies::get_movie_count(&conn).unwrap(), 2);
    }

    #[test]
    fn test_get_all_movies() {
        let (conn, _dir) = setup_db();
        movies::insert_movie(&conn, &sample_movie(1)).unwrap();
        movies::insert_movie(&conn, &sample_movie(2)).unwrap();
        movies::insert_movie(&conn, &sample_movie(3)).unwrap();

        let all = movies::get_all_movies(&conn).unwrap();
        assert_eq!(all.len(), 3);
    }

    // --- Movie summaries with sorting ---

    #[test]
    fn test_summaries_sorted_by_title_asc() {
        let (conn, _dir) = setup_db();
        let mut m1 = sample_movie(1);
        m1.title = "Z Movie".to_string();
        let mut m2 = sample_movie(2);
        m2.title = "A Movie".to_string();
        movies::insert_movie(&conn, &m1).unwrap();
        movies::insert_movie(&conn, &m2).unwrap();

        let results = movies::get_all_movie_summaries(&conn, "title", true, None, None).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "A Movie");
        assert_eq!(results[1].title, "Z Movie");
    }

    #[test]
    fn test_summaries_sorted_by_year_desc() {
        let (conn, _dir) = setup_db();
        movies::insert_movie(&conn, &sample_movie(1)).unwrap(); // year 2021
        movies::insert_movie(&conn, &sample_movie(2)).unwrap(); // year 2022

        let results = movies::get_all_movie_summaries(&conn, "year", false, None, None).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].year, Some(2022));
        assert_eq!(results[1].year, Some(2021));
    }

    #[test]
    fn test_summaries_filtered_by_genre() {
        let (conn, _dir) = setup_db();
        let mut m1 = sample_movie(1);
        m1.genres = Some(r#"["Action"]"#.to_string());
        let mut m2 = sample_movie(2);
        m2.genres = Some(r#"["Comedy"]"#.to_string());
        movies::insert_movie(&conn, &m1).unwrap();
        movies::insert_movie(&conn, &m2).unwrap();

        let results = movies::get_all_movie_summaries(&conn, "title", true, Some("Action"), None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Movie 1");
    }

    #[test]
    fn test_summaries_search_fallback() {
        let (conn, _dir) = setup_db();
        let mut m = sample_movie(1);
        m.title = "Inception".to_string();
        movies::insert_movie(&conn, &m).unwrap();

        let results = movies::get_all_movie_summaries(
            &conn, "title", true, None, Some("Inception"),
        ).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Inception");
    }

    // --- Settings ---

    #[test]
    fn test_settings_upsert() {
        let (conn, _dir) = setup_db();
        settings::set_setting(&conn, "test_key", "test_value").unwrap();
        let value = settings::get_setting(&conn, "test_key").unwrap();
        assert_eq!(value, "test_value");

        settings::set_setting(&conn, "test_key", "new_value").unwrap();
        let value = settings::get_setting(&conn, "test_key").unwrap();
        assert_eq!(value, "new_value");
    }

    #[test]
    fn test_settings_missing_key() {
        let (conn, _dir) = setup_db();
        let result = settings::get_setting(&conn, "nonexistent_key");
        assert!(result.is_err());
    }

    // --- Subtitles ---

    #[test]
    fn test_subtitle_insert_and_query() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        let movie_id = movies::insert_movie(&conn, &movie).unwrap();

        let sub = sample_subtitle(movie_id);
        subtitles::insert_subtitle(&conn, &sub).unwrap();

        let subs = subtitles::get_subtitles_for_movie(&conn, movie_id).unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].language, "zh-CN");
        assert_eq!(subs[0].rating, Some(9.0));
    }

    #[test]
    fn test_subtitle_count() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        let movie_id = movies::insert_movie(&conn, &movie).unwrap();

        subtitles::insert_subtitle(&conn, &sample_subtitle(movie_id)).unwrap();
        let mut sub2 = sample_subtitle(movie_id);
        sub2.language = "en".to_string();
        subtitles::insert_subtitle(&conn, &sub2).unwrap();

        let count = subtitles::get_subtitle_count_for_movie(&conn, movie_id).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_subtitle_delete() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        let movie_id = movies::insert_movie(&conn, &movie).unwrap();

        let sub = sample_subtitle(movie_id);
        let sub_id = subtitles::insert_subtitle(&conn, &sub).unwrap();

        subtitles::delete_subtitle(&conn, sub_id).unwrap();
        let subs = subtitles::get_subtitles_for_movie(&conn, movie_id).unwrap();
        assert!(subs.is_empty());
    }

    // --- Watchlist ---

    #[test]
    fn test_watchlist_crud() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        let movie_id = movies::insert_movie(&conn, &movie).unwrap();

        let item = WatchlistItem {
            id: 0,
            movie_id: Some(movie_id),
            tmdb_id: Some(101),
            status: "want_to_watch".to_string(),
            user_rating: None,
            notes: None,
            added_date: String::new(),
            watched_date: None,
        };
        let watch_id = watchlist::add_to_watchlist(&conn, &item).unwrap();
        assert!(watch_id > 0);

        let items = watchlist::get_watchlist(&conn, Some("want_to_watch")).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "want_to_watch");

        let mut updated = items[0].clone();
        updated.status = "watched".to_string();
        updated.user_rating = Some(8.5);
        watchlist::update_watchlist(&conn, &updated).unwrap();

        let watched = watchlist::get_watchlist(&conn, Some("watched")).unwrap();
        assert_eq!(watched.len(), 1);
        assert_eq!(watched[0].status, "watched");
        assert_eq!(watched[0].user_rating, Some(8.5));
    }

    #[test]
    fn test_watchlist_remove() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        let movie_id = movies::insert_movie(&conn, &movie).unwrap();

        let item = WatchlistItem {
            id: 0,
            movie_id: Some(movie_id),
            tmdb_id: None,
            status: "want_to_watch".to_string(),
            user_rating: None,
            notes: None,
            added_date: String::new(),
            watched_date: None,
        };
        let watch_id = watchlist::add_to_watchlist(&conn, &item).unwrap();

        watchlist::remove_from_watchlist(&conn, watch_id).unwrap();
        let items = watchlist::get_watchlist(&conn, Some("want_to_watch")).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_watchlist_workflow_summary_upsert() {
        let (conn, _dir) = setup_db();
        let movie = sample_movie(1);
        let movie_id = movies::insert_movie(&conn, &movie).unwrap();
        let stored_movie = movies::get_movie_by_id(&conn, movie_id).unwrap().unwrap();

        let item = WatchlistItem {
            id: 0,
            movie_id: Some(movie_id),
            tmdb_id: Some(101),
            status: "want_to_watch".to_string(),
            user_rating: None,
            notes: Some("Personal note".to_string()),
            added_date: String::new(),
            watched_date: None,
        };
        watchlist::add_to_watchlist(&conn, &item).unwrap();

        watchlist::upsert_workflow_summary(
            &conn,
            &stored_movie,
            "want_to_watch",
            "Workflow title\nSummary line",
        )
        .unwrap();

        watchlist::upsert_workflow_summary(
            &conn,
            &stored_movie,
            "want_to_watch",
            "Updated workflow title\nUpdated summary",
        )
        .unwrap();

        let saved = watchlist::get_watchlist_item_for_movie(&conn, movie_id)
            .unwrap()
            .unwrap();
        let extracted = watchlist::extract_workflow_summary(saved.notes.as_deref().unwrap()).unwrap();

        assert!(saved.notes.as_deref().unwrap().contains("Personal note"));
        assert_eq!(extracted, "Updated workflow title\nUpdated summary");
        assert_eq!(watchlist::get_watchlist(&conn, Some("want_to_watch")).unwrap().len(), 1);
    }
}
