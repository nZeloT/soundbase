use crate::error;

pub fn setup_db() -> error::Result<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>> {
    let manager = r2d2_sqlite::SqliteConnectionManager::file("./soundbase_test.db");
    let pool = r2d2::Pool::new(manager)?;
    let conn = pool.get()?;
    conn.execute_batch("\
         CREATE TABLE IF NOT EXISTS analytics (
            tmstp   INTEGER PRIMARY KEY,
            origin  VARCHAR(20) NOT NULL,
            kind    TINYINT,
            transition_src  TINYINT,
            transition_dst  TINYINT,
            playback_source TINYINT,
            playback_name   VARCHAR(25),
            playback_started BOOLEAN,
            song_raw        VARCHAR(60),
            song_title      VARCHAR(20),
            song_artist     VARCHAR(20),
            song_album      VARCHAR(20)
        );
        CREATE TABLE IF NOT EXISTS artists (
            artist_id       INTEGER PRIMARY KEY AUTOINCREMENT,
            artist_name     VARCHAR(30) NOT NULL,
            artist_spot_id  VARCHAR(22),

            CONSTRAINT artist_unique UNIQUE(artist_name)
        );
        CREATE TABLE IF NOT EXISTS albums (
            album_id        INTEGER PRIMARY KEY AUTOINCREMENT,
            artist_id       INTEGER NOT NULL,
            album_name      VARCHAR(30) NOT NULL,
            album_spot_id   VARCHAR(22),

            FOREIGN KEY(artist_id) REFERENCES artists(artist_id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS songs (
            song_id         INTEGER PRIMARY KEY AUTOINCREMENT,
            song_title      VARCHAR(20) NOT NULL,
            song_spot_id    VARCHAR(22),
            artist_id       INTEGER NOT NULL,
            album_id        INTEGER REFERENCES albums(album_id) ON DELETE CASCADE,
            is_faved        BOOLEAN,

            FOREIGN KEY(artist_id) REFERENCES artists(artist_id) ON DELETE CASCADE,
            CONSTRAINT song_unique UNIQUE(song_title,artist_id,album_id)
        );
        CREATE TABLE IF NOT EXISTS song_history (
            hist_id        INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id        INTEGER NOT NULL,
            change_origin  VARCHAR(20) NOT NULL,
            change_source  TINYINT NOT NULL,
            change_name    VARCHAR(20) NOT NULL,
            prev_state     TINYINT,
            new_state      TINYINT,

            FOREIGN KEY(song_id) REFERENCES songs(song_id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS albums_of_week (
            week_id         INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id        INTEGER NOT NULL,
            album_song_list_raw TEXT,
            source_name     VARCHAR(20) NOT NULL,
            source_comment  TEXT,
            source_date     VARCHAR(40),
            FOREIGN KEY(album_id) REFERENCES albums(album_id)
        );
        CREATE TABLE IF NOT EXISTS top_charts_of_week (
            week_song_id    INTEGER PRIMARY KEY AUTOINCREMENT,
            calendar_week   TINYINT,
            year            TINYINT,
            source_name     VARCHAR(20) NOT NULL,
            song_id         INTEGER NOT NULL,
            song_position   TINYINT,

            FOREIGN KEY(song_id) REFERENCES songs(song_id)
        );
    ")?;
    Ok(pool)
}