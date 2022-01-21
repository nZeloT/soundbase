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

use crate::db_new::{DbApi, DbPool, Result};
use crate::db_new::db_error::DbError;
use crate::db_new::{FindByFavedStatus, FindById};
use crate::db_new::models::{Track, NewTrack, Album};
use crate::db_new::schema::*;
use crate::model::UniversalTrackId;

pub trait TrackDb: FindById<Track> + FindByFavedStatus<Track> + Sync {
    fn new_track(&self, title: &str, album_id: i32, duration_ms: i32, is_faved: bool) -> Result<Track>;
    fn new_full_track(&self, new_track: NewTrack) -> Result<Track>;
    fn find_track_by_album(&self, album : &Album, name : &str) -> Result<Option<Track>>;
    fn find_track_by_universal_id(&self, uni_id : &UniversalTrackId) -> Result<Option<Track>>;
    fn load_tracks_for_album(&self, album : &Album) -> Result<Vec<Track>>;
    fn set_faved_state(&self, track_id : i32, now_faved : bool) -> Result<()>;
}

impl TrackDb for DbApi {
    fn new_track(&self, title: &str, album_id: i32, duration_ms: i32, is_faved: bool) -> Result<Track> {
        let new_track = NewTrack {
            title,
            album_id,
            disc_number: None,
            track_number: None,
            duration_ms,
            is_faved,
            local_file: None,
            spot_id: None,
        };
        self.new_full_track(new_track)
    }

    fn new_full_track(&self, new_track: NewTrack) -> Result<Track> {
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::insert_into(tracks::table)
                    .values(&new_track)
                    .get_result(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn find_track_by_album(&self, album: &Album, name: &str) -> Result<Option<Track>> {
        match self.0.get() {
            Ok(conn) => {
                let result = Track::belonging_to(album)
                    .filter(tracks::title.ilike(name))
                    .first(&conn)
                    .optional();
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn find_track_by_universal_id(&self, uni_id: &UniversalTrackId) -> Result<Option<Track>> {
        match uni_id {
            UniversalTrackId::Spotify(spot) => {
                match self.0.get() {
                    Ok(conn) => {
                        let result = tracks::table
                            .filter(tracks::spot_id.like(spot))
                            .first(&conn)
                            .optional();
                        match result {
                            Ok(v) => Ok(v),
                            Err(e) => Err(DbError::from(e))
                        }
                    },
                    Err(_) => Err(DbError::pool_timeout())
                }
            },
            UniversalTrackId::Database(id) => Ok(Some(self.find_by_id(*id)?))
        }
    }
    
    fn load_tracks_for_album(&self, album: &Album) -> Result<Vec<Track>> {
        match self.0.get() {
            Ok(conn) => {
                let result = Track::belonging_to(album)
                    .load::<Track>(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn set_faved_state(&self, id: i32, now_faved: bool) -> Result<()> {
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::update(
                    tracks::table.filter(tracks::track_id.eq(id))
                ).set(tracks::is_faved.eq(now_faved))
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

impl FindById<Track> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<Track> {
        use crate::db_new::schema::tracks::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = tracks
                    .find(id)
                    .first(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl FindByFavedStatus<Track> for DbApi {
    fn find_only_faved(&self, offset: i64, limit: i64) -> Result<Vec<Track>> {
        _find_by_fav_status(&self.0, true, offset, limit)
    }

    fn find_only_unfaved(&self, offset: i64, limit: i64) -> Result<Vec<Track>> {
        _find_by_fav_status(&self.0, false, offset, limit)
    }
}

fn _find_by_fav_status(pool : &DbPool, faved : bool, offset : i64, limit : i64) -> Result<Vec<Track>> {
    use crate::db_new::schema::tracks::dsl::*;
    match pool.get() {
        Ok(conn) => {
            let results = tracks
                .filter(is_faved.eq(faved))
                .limit(limit)
                .offset(offset)
                .load::<Track>(&conn);

            match results {
                Ok(values) => Ok(values),
                Err(e) => Err(DbError::from(e))
            }
        },
        Err(_) => Err(DbError::pool_timeout())
    }
}