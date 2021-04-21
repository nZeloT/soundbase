use crate::song_like::{Song, SongSource, SongState, SongMetadata};
use crate::analytics::{Metadata, PageChange, PlaybackChange, SongChange};
use crate::error;

pub trait SongDB {
    fn get_state(&mut self, song: &Song) -> error::Result<SongState>;

    fn fav_song(&mut self, song: &Song, meta: &SongMetadata) -> error::Result<SongState>;
    fn unfav_song(&mut self, song: &Song, meta: &SongMetadata) -> error::Result<SongState>;
}

pub trait AnalyticsDB {
    fn store_page_change(&mut self, meta: &Metadata, change: &PageChange) -> error::Result<()>;
    fn store_playback_change(&mut self, meta: &Metadata, playback: &PlaybackChange) -> error::Result<()>;
    fn store_song_change(&mut self, meta: &Metadata, song: &SongChange) -> error::Result<()>;
}

trait SongHistDB {
    fn store_song_hist_entry(&mut self, song: &Song, meta: &SongMetadata, new_state: bool) -> error::Result<()>;
}

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
        CREATE TABLE IF NOT EXISTS songs (
            song_id         INTEGER PRIMARY KEY AUTOINCREMENT,
            title           VARCHAR(20) NOT NULL,
            artist          VARCHAR(20) NOT NULL,
            album           VARCHAR(20),
            current_state   BOOLEAN,

            CONSTRAINT song_unique UNIQUE(title,artist,album)
        );
        CREATE TABLE IF NOT EXISTS song_history (
            hist_id        INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id        INTEGER REFERENCES songs(song_id) NOT DEFERRABLE,
            change_origin  VARCHAR(20) NOT NULL,
            change_source  TINYINT NOT NULL,
            change_name    VARCHAR(20) NOT NULL,
            prev_state     TINYINT,
            new_state      TINYINT
        );
    ")?;
    Ok(pool)
}

impl SongDB for r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> {
    fn get_state(&mut self, song: &Song) -> error::Result<SongState> {
        let mut stmt = self.prepare_cached("SELECT current_state FROM songs WHERE title = ? AND artist = ? AND album = ?")?;
        let mut rows = stmt.query(rusqlite::params![
            &song.title,
            &song.artist,
            &song.album
        ])?;

        let potential_song = rows.next()?;
        match potential_song {
            Some(song) => {
                println!("\tFound queried song in DB.");
                let song_state: u32 = song.get(0)?;
                if song_state == 0 {
                    Ok(SongState::NotFaved)
                } else {
                    Ok(SongState::Faved)
                }
            }
            None => Ok(SongState::NotFound)
        }
    }

    fn fav_song(&mut self, song: &Song, meta: &SongMetadata) -> error::Result<SongState> {
        let current_state = self.get_state(song)?;
        match current_state {
            SongState::NotFound => {
                {
                    let mut stmt = self.prepare_cached("INSERT INTO songs (title,artist,album,current_state) VALUES (?,?,?,?)")?;
                    stmt.execute(rusqlite::params![
                       &song.title,
                        &song.artist,
                        &song.album,
                        1
                    ])?;
                };
                self.store_song_hist_entry(song, meta, true)?;
                Ok(SongState::NowFaved)
            }
            SongState::NotFaved => {
                {
                    let mut stmt = self.prepare_cached("UPDATE songs SET current_state = 1 WHERE title = ? AND artist = ? AND album = ?")?;
                    stmt.execute(rusqlite::params![
                       &song.title,
                        &song.artist,
                        &song.album
                    ])?;
                };
                self.store_song_hist_entry(song, meta, true)?;
                Ok(SongState::NowFaved)
            }
            SongState::Faved => Ok(SongState::Faved),
            SongState::NowFaved | SongState::NowNotFaved => Err(error::SoundbaseError { http_code: tide::StatusCode::InternalServerError, msg: String::from("Received NowFaved or NowNotFaved from get_state(song)!") })
        }
    }

    fn unfav_song(&mut self, song: &Song, meta: &SongMetadata) -> error::Result<SongState> {
        let current_state = self.get_state(song)?;
        match current_state {
            SongState::NotFound | SongState::NotFaved => { Ok(SongState::NotFaved) }
            SongState::Faved => {
                {
                    let mut stmt = self.prepare_cached("UPDATE songs SET current_state = 0 WHERE title = ? AND artist = ? AND album = ?")?;
                    stmt.execute(rusqlite::params![
                        &song.title,
                        &song.artist,
                        &song.album
                    ])?;
                };
                self.store_song_hist_entry(song, meta, false)?;

                Ok(SongState::NowNotFaved)
            }
            SongState::NowFaved | SongState::NowNotFaved => Err(error::SoundbaseError { http_code: tide::StatusCode::InternalServerError, msg: String::from("Received NowFaved or NowNotFaved from get_state(song)!") })
        }
    }
}

impl AnalyticsDB for r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> {
    fn store_page_change(&mut self, meta: &Metadata, change: &PageChange) -> error::Result<()> {
        let mut stmt = self.prepare_cached("INSERT INTO analytics (tmstp,origin,kind,transition_src,transition_dst) VALUES (?,?,?,?,?)")?;

        let kind: u8 = meta.kind.into();
        let src: u8 = change.src.into();
        let dst: u8 = change.dst.into();

        stmt.execute(rusqlite::params![
            meta.tmstp.timestamp_millis(),
            &meta.origin,
            kind,
            src,
            dst
        ])?;

        println!("\tPage Change stored.");
        Ok(())
    }

    fn store_playback_change(&mut self, meta: &Metadata, playback: &PlaybackChange) -> error::Result<()> {
        let mut stmt = self.prepare_cached("INSERT INTO analytics (tmstp,origin,kind,playback_source,playback_name,playback_started) VALUES (?,?,?,?,?,?)")?;
        let kind: u8 = meta.kind.into();
        let source: u8 = playback.source.into();
        stmt.execute(rusqlite::params![
            meta.tmstp.timestamp_millis(),
            &meta.origin,
            kind,
            source,
            &playback.name,
            playback.started
        ])?;
        println!("\tPlayback Change stored.");
        Ok(())
    }

    fn store_song_change(&mut self, meta: &Metadata, song: &SongChange) -> error::Result<()> {
        let mut stmt = self.prepare_cached("INSERT INTO analytics (tmstp,origin,kind,song_raw,song_title,song_artist,song_album) VALUES (?,?,?,?,?,?,?)")?;
        let kind: u8 = meta.kind.into();
        stmt.execute(rusqlite::params![
            meta.tmstp.timestamp_millis(),
            &meta.origin,
            kind,

            &song.raw_meta,
            &song.title,
            &song.artist,
            &song.album
        ])?;
        println!("\tSong Change stored.");
        Ok(())
    }
}

impl SongHistDB for r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> {
    fn store_song_hist_entry(&mut self, song: &Song, meta: &SongMetadata, new_state: bool) -> error::Result<()> {
        println!("\tStored in Hist DB!");

        let mut stmt = self.prepare_cached("SELECT song_id FROM songs WHERE title = ? AND artist = ? AND album = ?")?;
        let mut rows = stmt.query(rusqlite::params![&song.title, &song.artist, &song.album])?;
        match rows.next()? {
            Some(song_row) => {
                let song_id :u64 = song_row.get(0)?;
                let mut stmt = self.prepare_cached("INSERT INTO song_history (song_id,change_origin,change_source,change_name,prev_state,new_state) VALUES(?,?,?,?,?,?)")?;
                stmt.execute(rusqlite::params![
                    song_id,
                    &meta.origin,
                    &meta.source.source_kind,
                    &meta.source.source_name,
                    !new_state,
                    new_state
                ])?;
            },
            None => {}
        }

        Ok(())
    }
}