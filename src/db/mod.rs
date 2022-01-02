/*
 * Copyright 2021 nzelot<leontsteiner@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::path::Path;

pub mod db_error;
pub mod artist;
pub mod album;
pub mod album_of_week;
pub mod song;
pub mod top_of_the_week;
mod util;

type Result<R> = std::result::Result<R, db_error::DbError>;
pub(crate) type DbConn    = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;
pub(crate) type DbPool    = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

pub trait Load<R> {
    fn load(&self, id: u64) -> Result<R>;
}

pub trait FollowForeignReference<O, D> {
    fn follow_reference(&self, to_follow: &O) -> Result<D>;
}

pub trait FindUnique<R, Q> {
    fn find_unique(&self, query: Q) -> Result<Option<R>>;
}

pub trait Save<R> {
    fn save(&self, to_save: &mut R) -> Result<()>;
}

pub trait Delete<R> {
    fn delete(&self, to_delete: &R) -> Result<()>;
}

pub fn initialize_db<P : AsRef<Path>>(path : P) -> Result<DbPool> {
    let manager = r2d2_sqlite::SqliteConnectionManager::file(path);
    let pool = r2d2::Pool::builder().max_size(5).build(manager)?;
    let conn = pool.get()?;
    conn.execute_batch("\
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
        CREATE TABLE IF NOT EXISTS albums_of_week (
            aow_id         INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id        INTEGER NOT NULL,
            album_song_list_raw TEXT,
            source_name     VARCHAR(20) NOT NULL,
            source_comment  TEXT,
            source_date     VARCHAR(40),
            year            INTEGER,
            week            INTEGER,
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