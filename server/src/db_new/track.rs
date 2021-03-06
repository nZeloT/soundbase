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
use crate::db_new::models::{Track, NewTrack, Album, Artist, TrackArtists};
use crate::db_new::schema::*;
use crate::model::{RequestPage, UniversalId};

pub trait TrackDb: FindById<Track> + FindByFavedStatus<Track> + SetFavedState<Track> + Sync {
    fn new_full_track(&self, new_track: NewTrack) -> Result<Track>;
    fn find_track_by_album(&self, album : &Album, name : &str) -> Result<Option<Track>>;
    fn find_track_by_universal_id(&self, uni_id : &UniversalId) -> Result<Option<Track>>;
    fn load_tracks_for_album(&self, album : &Album) -> Result<Vec<Track>>;
    fn load_fav_tracks_for_artist(&self, artist : &Artist, page : &RequestPage) -> Result<Vec<Track>>;
    fn load_tracks(&self, page : &RequestPage) -> Result<Vec<Track>>;
}

impl TrackDb for DbApi {

    fn new_full_track(&self, new_track: NewTrack) -> Result<Track> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(tracks::table)
            .values(&new_track)
            .get_result(&conn);
        Ok(result?)
    }

    fn find_track_by_album(&self, album: &Album, name: &str) -> Result<Option<Track>> {
        let conn = self.0.get()?;
        let result = Track::belonging_to(album)
            .filter(tracks::title.ilike(name))
            .first(&conn)
            .optional();
        Ok(result?)
    }

    fn find_track_by_universal_id(&self, uni_id: &UniversalId) -> Result<Option<Track>> {
        match uni_id {
            UniversalId::Spotify(spot) => {
                let conn = self.0.get()?;
                let result = tracks::table
                    .filter(tracks::spot_id.like(spot))
                    .first(&conn)
                    .optional();
                Ok(result?)
            },
            UniversalId::Database(id) => self.find_by_id(*id)
        }
    }
    
    fn load_tracks_for_album(&self, album: &Album) -> Result<Vec<Track>> {
        let conn = self.0.get()?;
        let result = Track::belonging_to(album)
            .load::<Track>(&conn);
        Ok(result?)
    }

    fn load_fav_tracks_for_artist(&self, artist: &Artist, page: &RequestPage) -> Result<Vec<Track>> {
        let conn = self.0.get()?;
        let tracks = TrackArtists::belonging_to(artist).select(track_artist::track_id);
        let results = tracks::table
            .filter(tracks::track_id.eq(diesel::dsl::any(tracks)))
            .filter(tracks::is_faved.eq(true))
            .offset(page.offset())
            .limit(page.limit())
            .load::<Track>(&conn);
        Ok(results?)
    }

    fn load_tracks(&self, page : &RequestPage) -> Result<Vec<Track>> {
        let conn = self.0.get()?;
        let result = tracks::table
            .offset(page.offset())
            .limit(page.limit())
            .load::<Track>(&conn);
        Ok(result?)
    }
}

impl FindById<Track> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<Option<Track>> {
        use crate::db_new::schema::tracks::dsl::*;
        let conn = self.0.get()?;
        let result = tracks
            .find(id)
            .first(&conn)
            .optional();
        Ok(result?)
    }

    fn find_by_ids(&self, ids: Vec<i32>) -> Result<Vec<Track>> {
        use diesel::dsl::any;
        let conn = self.0.get()?;
        let result = tracks::table
            .filter(tracks::track_id.eq(any(ids)))
            .load::<Track>(&conn);
        Ok(result?)
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

impl SetFavedState<Track> for DbApi {
    fn set_faved_state(&self, id: i32, now_faved: bool) -> Result<()> {
        let conn = self.0.get()?;
        let updated = diesel::update(
            tracks::table.filter(tracks::track_id.eq(id))
        ).set(tracks::is_faved.eq(now_faved))
            .execute(&conn)?;

        if updated == 1 {
            Ok(())
        }else{
            Err(DbError::Update(format!("Failed to set track {} to fav state {}", id, now_faved)))
        }
    }
}

fn _find_by_fav_status(pool : &DbPool, faved : bool, offset : i64, limit : i64) -> Result<Vec<Track>> {
    use crate::db_new::schema::tracks::dsl::*;
    let conn = pool.get()?;
    let results = tracks
        .filter(is_faved.eq(faved))
        .limit(limit)
        .offset(offset)
        .load::<Track>(&conn);
    Ok(results?)
}