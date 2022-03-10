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

use crate::db_new::{DbApi, DbError, DbPool, Result, SetFavedState};
use crate::db_new::{FindByFavedStatus, FindById};
use crate::db_new::models::{Artist, NewArtist};
use crate::db_new::schema::*;
use crate::model::{RequestPage, UniversalId};

pub trait ArtistDb: FindById<Artist> + FindByFavedStatus<Artist> + SetFavedState<Artist> {
    fn new_full_artist(&self, new_artist: NewArtist) -> Result<Artist>;
    fn find_artist_by_name(&self, name: &str) -> Result<Option<Artist>>;
    fn find_artist_by_universal_id(&self, id : &UniversalId) -> Result<Option<Artist>>;
    fn load_artists(&self, page : &RequestPage) -> Result<Vec<Artist>>;
}

impl ArtistDb for DbApi {

    fn new_full_artist(&self, new_artist: NewArtist) -> Result<Artist> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(artists::table)
            .values(&new_artist)
            .get_result(&conn);
        Ok(result?)
    }


    fn find_artist_by_name(&self, name: &str) -> Result<Option<Artist>> {
        let conn = self.0.get()?;
        let result = artists::table
            .filter(artists::name.eq(name))
            .first(&conn)
            .optional();
        Ok(result?)
    }

    fn find_artist_by_universal_id(&self, id: &UniversalId) -> Result<Option<Artist>> {
        match id {
            UniversalId::Spotify(spot_id) => {
                let conn = self.0.get()?;
                let result = artists::table
                    .filter(artists::spot_id.like(spot_id))
                    .first(&conn)
                    .optional();
                Ok(result?)
            },
            UniversalId::Database(artist_id) => self.find_by_id(*artist_id)
        }
    }

    fn load_artists(&self, page: &RequestPage) -> Result<Vec<Artist>> {
        let conn = self.0.get()?;
        let result = artists::table.offset(page.offset()).limit(page.limit()).load::<Artist>(&conn);
        Ok(result?)
    }
}

impl FindById<Artist> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<Option<Artist>> {
        use crate::db_new::schema::artists::dsl::*;
        let conn = self.0.get()?;
        let result = artists
            .find(id)
            .first(&conn)
            .optional();
        Ok(result?)
    }

    fn find_by_ids(&self, ids : Vec<i32>) -> Result<Vec<Artist>> {
        use diesel::dsl::any;
        let conn = self.0.get()?;
        let result = artists::table
            .filter(artists::artist_id.eq(any(ids)))
            .load::<Artist>(&conn);
        Ok(result?)
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

impl SetFavedState<Artist> for DbApi {
    fn set_faved_state(&self, artist_id: i32, now_faved: bool) -> Result<()> {
        let conn = self.0.get()?;
        let updated = diesel::update(
            artists::table.filter(artists::artist_id.eq(artist_id))
        ).set(artists::is_faved.eq(now_faved))
            .execute(&conn)?;
        if updated == 1 {
            Ok(())
        }else{
            Err(DbError::Update(format!("Failed to set artist {} to fav state {}!", artist_id, now_faved)))
        }
    }
}

fn _find_by_fav_status(pool: &DbPool, faved: bool, offset: i64, limit: i64) -> Result<Vec<Artist>> {
    use crate::db_new::schema::artists::dsl::*;
    let conn = pool.get()?;
    let results = artists
        .filter(is_faved.eq(faved))
        .limit(limit)
        .offset(offset)
        .load::<Artist>(&conn);
    Ok(results?)
}