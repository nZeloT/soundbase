/*
 * Copyright 2022 nzelot<leontsteiner@gmail.com>
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

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use thiserror::Error;
use url::Url;

use crate::db_new::album::AlbumDb;
use crate::db_new::album_artist::AlbumArtistsDb;
use crate::db_new::artist::ArtistDb;
use crate::db_new::models::{Album, Artist, NewAlbum, NewArtist, NewTrack, Track};
use crate::db_new::track::TrackDb;

mod schema;
pub mod models;
pub mod genre;
pub mod artist;
pub mod artist_genre;
pub mod album;
pub mod album_artist;
pub mod track;
pub mod track_artist;
pub mod album_of_week;
pub mod charts_of_week;
pub mod track_fav_proposal;

type DbPool = Pool<ConnectionManager<PgConnection>>;

type Result<R> = std::result::Result<R, DbError>;

#[derive(Error, Debug)]
pub enum DbError{
    #[error("DB connection error: {0}")]
    Connection(#[from] r2d2::Error),

    #[error("DB sql execution error: {0}")]
    Sql(#[from] diesel::result::Error),
    
    #[error("DB update failed: {0}")]
    Update(String),
    
    #[error("DB delete failed: {0}")]
    Delete(String)
}

pub trait FindById<T> {
    fn find_by_id(&self, id: i32) -> Result<T>;
}

pub trait FindByFavedStatus<T> {
    fn find_only_faved(&self, offset: i64, limit: i64) -> Result<Vec<T>>;
    fn find_only_unfaved(&self, offset: i64, limit: i64) -> Result<Vec<T>>;
}

pub trait UpdateSingle<T> {
    fn update(&self, to_update: &T) -> Result<()>;
}

#[derive(Clone)]
pub struct DbApi(DbPool);
impl DbApi {

    pub fn new(url: Url) -> Self {
        let pool = init_db(url);
        DbApi(pool)
    }

    pub fn get_or_create_artist<'a, Fn>(&self, name: &str, create_fn: Fn) -> Result<Artist>
        where Fn: FnOnce() -> NewArtist<'a> {
        let api: &dyn ArtistDb = self;
        let opt = api.find_artist_by_name(name)?;
        match opt {
            Some(artist) => Ok(artist),
            None => Ok(api.new_full_artist(create_fn())?)
        }
    }

    pub fn get_or_create_album<'a, Fn>(&self, artist: &Artist, name: &str, create_fn: Fn) -> Result<Album>
        where Fn: FnOnce() -> NewAlbum<'a> {
        let api: &dyn AlbumDb = self;
        let opt = api.find_by_artist_and_name(artist, name)?;
        match opt {
            Some(val) => Ok(val),
            None => {
                let new_album = api.new_full_album(create_fn())?;

                //link artist with album
                let api : &dyn AlbumArtistsDb = self;
                let _ = api.new_album_artist(artist.artist_id, new_album.album_id)?;

                Ok(new_album)
            }
        }
    }

    pub fn get_or_create_track<'a, Fn>(&self, album : &Album, name : &str, create_fn : Fn) -> Result<Track>
        where Fn: FnOnce() -> NewTrack<'a> {
        let api : &dyn TrackDb = self;
        let opt = api.find_track_by_album(album, name)?;
        match opt {
            Some(v) => Ok(v),
            None => {
                let new_track = api.new_full_track(create_fn())?;
                Ok(new_track)
            }
        }
    }
}

fn init_db(url: Url) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("DB connection failed!");

    //TODO run migrations

    pool
}