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

use diesel::prelude::*;

use crate::db_new::{DbApi, Result};
use crate::db_new::db_error::DbError;
use crate::db_new::models::{Album, Artist, AlbumArtists, NewAlbumArtists};
use crate::db_new::schema::*;
use crate::db_new::UpdateSingle;

pub trait AlbumArtistsDb : UpdateSingle<AlbumArtists> {
    fn new_album_artist(&self, artist_id : i32, album_id : i32) -> Result<AlbumArtists>;
    fn load_albums_for_artist(&self, artist: &Artist, offset : i64, limit : i64) -> Result<Vec<Album>>;
    fn load_artists_for_album(&self, album: &Album, offset : i64, limit : i64) -> Result<Vec<Artist>>;
}

impl AlbumArtistsDb for DbApi {
    fn new_album_artist(&self, artist_id: i32, album_id: i32) -> Result<AlbumArtists> {
        match self.0.get() {
            Ok(conn) => {
                let new_aa = NewAlbumArtists{
                    album_id,
                    artist_id
                };
                let result = diesel::insert_into(album_artists::table)
                    .values(&new_aa)
                    .get_result(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn load_albums_for_artist(&self, artist: &Artist, offset: i64, limit: i64) -> Result<Vec<Album>> {
        use crate::db_new::schema::album_artists::dsl::*;
        use diesel::dsl::any;

        match self.0.get() {
            Ok(conn) => {
                let album_ids = AlbumArtists::belonging_to(artist).select(album_id);
                let result = albums::table
                    .filter(albums::album_id.eq(any(album_ids)))
                    .limit(limit)
                    .offset(offset)
                    .load::<Album>(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn load_artists_for_album(&self, album: &Album, offset: i64, limit: i64) -> Result<Vec<Artist>> {
        use crate::db_new::schema::album_artists::dsl::*;
        use diesel::dsl::any;

        match self.0.get() {
            Ok(conn) => {
                let artist_ids = AlbumArtists::belonging_to(album).select(artist_id);
                let result = artists::table
                    .filter(artists::artist_id.eq(any(artist_ids)))
                    .limit(limit)
                    .offset(offset)
                    .load::<Artist>(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl UpdateSingle<AlbumArtists> for DbApi {
    fn update(&self, to_update: &AlbumArtists) -> Result<()> {
        use crate::db_new::schema::album_artists::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::update(album_artists)
                    .set(to_update)
                    .execute(&conn);
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}