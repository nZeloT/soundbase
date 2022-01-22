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
use crate::db_new::models::{Track, Artist, TrackArtists, NewTrackArtists};
use crate::db_new::schema::*;
use crate::model::Page;

pub trait TrackArtistsDb {
    fn new_track_artist(&self, track_id : i32, artist_id : i32) -> Result<TrackArtists>;
    fn new_track_artist_if_missing(&self, track_id : i32, artist_id : i32) -> Result<TrackArtists>;
    fn load_track_for_artist(&self, artist: &Artist, page : Page) -> Result<Vec<Track>>;
    fn load_artists_for_track(&self, track : &Track, page : Page) -> Result<Vec<Artist>>;
}

impl TrackArtistsDb for DbApi {
    fn new_track_artist(&self, track_id: i32, artist_id: i32) -> Result<TrackArtists> {
        match self.0.get() {
            Ok(conn) => {
                let new_ta = NewTrackArtists{
                    track_id,
                    artist_id
                };
                let result = diesel::insert_into(track_artist::table)
                    .values(&new_ta)
                    .get_result(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn new_track_artist_if_missing(&self, track_id: i32, artist_id: i32) -> Result<TrackArtists> {
        let option = match self.0.get() {
            Ok(conn) => {
                track_artist::table
                    .filter(track_artist::track_id.eq(track_id))
                    .filter(track_artist::artist_id.eq(artist_id))
                    .first(&conn)
                    .optional()?
            },
            Err(_) => return Err(DbError::pool_timeout())
        };

        match option {
            Some(link) => Ok(link),
            None => self.new_track_artist(track_id, artist_id)
        }
    }


    fn load_track_for_artist(&self, artist: &Artist, page: Page) -> Result<Vec<Track>> {
        use crate::db_new::schema::track_artist::dsl::*;
        use diesel::dsl::any;

        match self.0.get() {
            Ok(conn) => {
                let track_ids = TrackArtists::belonging_to(artist).select(track_id);
                let result = tracks::table
                    .filter(tracks::track_id.eq(any(track_ids)))
                    .limit(page.limit())
                    .offset(page.offset())
                    .load::<Track>(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn load_artists_for_track(&self, track: &Track, page: Page) -> Result<Vec<Artist>> {
        use crate::db_new::schema::track_artist::dsl::*;
        use diesel::dsl::any;

        match self.0.get() {
            Ok(conn) => {
                let artist_ids = TrackArtists::belonging_to(track).select(artist_id);
                let result = artists::table
                    .filter(artists::artist_id.eq(any(artist_ids)))
                    .offset(page.offset())
                    .limit(page.limit())
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