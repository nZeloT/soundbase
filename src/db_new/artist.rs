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
use crate::db_new::{FindByFavedStatus, FindById, UpdateSingle};
use crate::db_new::models::{Artist, NewArtist};
use crate::db_new::schema::*;
use crate::model::UniversalId;

pub trait ArtistDb: FindById<Artist> + FindByFavedStatus<Artist> + UpdateSingle<Artist> {
    fn new_artist(&self, name: &str, spot_id: Option<String>) -> Result<Artist>;
    fn new_full_artist(&self, new_artist: NewArtist) -> Result<Artist>;
    fn find_artist_by_name(&self, name: &str) -> Result<Option<Artist>>;
    fn find_artist_by_universal_id(&self, id : &UniversalId) -> Result<Option<Artist>>;
}

impl ArtistDb for DbApi {
    fn new_artist(&self, name: &str, spot_id: Option<String>) -> Result<Artist> {
        self.new_full_artist(NewArtist {
            name,
            spot_id,
        })
    }

    fn new_full_artist(&self, new_artist: NewArtist) -> Result<Artist> {
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::insert_into(artists::table)
                    .values(&new_artist)
                    .get_result(&conn);
                match result {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }


    fn find_artist_by_name(&self, name: &str) -> Result<Option<Artist>> {
        match self.0.get() {
            Ok(conn) => {
                let result = artists::table
                    .filter(artists::name.eq(name))
                    .first(&conn)
                    .optional();
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn find_artist_by_universal_id(&self, id: &UniversalId) -> Result<Option<Artist>> {
        match id {
            UniversalId::Spotify(spot_id) => {
                match self.0.get() {
                    Ok(conn) => {
                        let result = artists::table
                            .filter(artists::spot_id.like(spot_id))
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
            UniversalId::Database(artist_id) => Ok(Some(self.find_by_id(*artist_id)?))
        }
    }
}

impl FindById<Artist> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<Artist> {
        use crate::db_new::schema::artists::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = artists
                    .find(id)
                    .first(&conn);
                match result {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl FindByFavedStatus<Artist> for DbApi {
    fn find_only_faved(&self, offset: i64, limit: i64) -> Result<Vec<Artist>> {
        _find_by_fav_status(&self.0, true, offset, limit)
    }

    fn find_only_unfaved(&self, offset: i64, limit: i64) -> Result<Vec<Artist>> {
        _find_by_fav_status(&self.0, false, offset, limit)
    }
}

impl UpdateSingle<Artist> for DbApi {
    fn update(&self, to_update: &Artist) -> Result<()> {
        use crate::db_new::schema::artists::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::update(artists)
                    .set(to_update)
                    .execute(&conn);
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

fn _find_by_fav_status(pool: &DbPool, faved: bool, offset: i64, limit: i64) -> Result<Vec<Artist>> {
    use crate::db_new::schema::artists::dsl::*;
    match pool.get() {
        Ok(conn) => {
            let results = artists
                .filter(is_faved.eq(faved))
                .limit(limit)
                .offset(offset)
                .load::<Artist>(&conn);

            match results {
                Ok(values) => Ok(values),
                Err(e) => Err(DbError::from(e))
            }
        }
        Err(_) => Err(DbError::pool_timeout())
    }
}