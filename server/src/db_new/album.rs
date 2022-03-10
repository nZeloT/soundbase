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
use itertools::Itertools;

use crate::db_new::{DbApi, DbError, DbPool, Result, SetFavedState};
use crate::db_new::{FindByFavedStatus, FindById};
use crate::db_new::models::{Album, AlbumArtists, AlbumOfWeek, Artist, NewAlbum, Track};
use crate::db_new::schema::*;
use crate::model::{RequestPage, UniversalId};

pub trait AlbumDb: FindById<Album> + FindByFavedStatus<Album> + SetFavedState<Album> + Sync {
    fn new_full_album(&self, new_album: NewAlbum) -> Result<Album>;
    fn find_by_artist_and_name(&self, artist: &Artist, name: &str) -> Result<Option<Album>>;
    fn find_by_universal_id(&self, id : &UniversalId) -> Result<Option<Album>>;
    fn load_album_for_track(&self, track: &Track) -> Result<Album>;
    fn load_albums_for_tracks(&self, tracks : &[Track]) -> Result<Vec<Album>>;
    fn load_album_for_aow(&self, aow: &AlbumOfWeek) -> Result<Album>;
    fn load_albums(&self, page : &RequestPage) -> Result<Vec<Album>>;
    fn set_was_aow(&self, album: &Album, was_aow: bool) -> Result<Album>;
}

impl AlbumDb for DbApi {
    fn new_full_album(&self, new_album: NewAlbum) -> Result<Album> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(albums::table)
            .values(&new_album)
            .get_result(&conn);
        Ok(result?)
    }

    fn find_by_artist_and_name(&self, artist: &Artist, name: &str) -> Result<Option<Album>> {
        use diesel::dsl::any;
        let conn = self.0.get()?;
        let album_ids = AlbumArtists::belonging_to(artist).select(album_artists::album_id);
        let result = albums::table
            .filter(albums::album_id.eq(any(album_ids)))
            .filter(albums::name.ilike(name))
            .first(&conn)
            .optional();
        Ok(result?)
    }

    fn find_by_universal_id(&self, id: &UniversalId) -> Result<Option<Album>> {
        match id {
            UniversalId::Spotify(spot_id) => {
                let conn = self.0.get()?;
                let result = albums::table
                    .filter(albums::spot_id.like(spot_id))
                    .first(&conn)
                    .optional();
                Ok(result?)
            },
            UniversalId::Database(album_id) => self.find_by_id(*album_id)
        }
    }


    fn load_album_for_track(&self, track: &Track) -> Result<Album> {
        Ok(self.find_by_id(track.album_id)?.unwrap())
    }

    fn load_album_for_aow(&self, aow: &AlbumOfWeek) -> Result<Album> {
        Ok(self.find_by_id(aow.album_id)?.unwrap())
    }

    fn load_albums_for_tracks(&self, tracks: &[Track]) -> Result<Vec<Album>> {
        let ids = tracks.iter().map(|t| t.album_id).unique().collect_vec();
        self.find_by_ids(ids)
    }

    fn load_albums(&self, page: &RequestPage) -> Result<Vec<Album>> {
        let conn = self.0.get()?;
        let results = albums::table
            .offset(page.offset())
            .limit(page.limit())
            .load::<Album>(&conn);
        Ok(results?)
    }

    fn set_was_aow(&self, album: &Album, was_aow: bool) -> Result<Album> {
        let conn = self.0.get()?;
        let result = diesel::update(
            albums::table.filter(albums::album_id.eq(album.album_id))
        ).set(albums::was_aow.eq(was_aow))
            .get_result(&conn);
        Ok(result?)
    }
}

impl FindById<Album> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<Option<Album>> {
        use crate::db_new::schema::albums::dsl::*;
        let conn = self.0.get()?;
        let result = albums
            .find(id)
            .first(&conn)
            .optional();
        Ok(result?)
    }

    fn find_by_ids(&self, ids : Vec<i32>) -> Result<Vec<Album>> {
        use diesel::dsl::any;
        let conn = self.0.get()?;
        let result = albums::table
            .filter(albums::album_id.eq(any(ids)))
            .load::<Album>(&conn);
        Ok(result?)
    }
}

impl FindByFavedStatus<Album> for DbApi {
    fn find_only_faved(&self, offset: i64, limit: i64) -> Result<Vec<Album>> {
        _find_by_fav_status(&self.0, true, offset, limit)
    }

    fn find_only_unfaved(&self, offset: i64, limit: i64) -> Result<Vec<Album>> {
        _find_by_fav_status(&self.0, false, offset, limit)
    }
}

impl SetFavedState<Album> for DbApi {
    fn set_faved_state(&self, album_id: i32, now_faved: bool) -> Result<()> {
        let conn = self.0.get()?;
        let updated = diesel::update(
            albums::table.filter(albums::album_id.eq(album_id))
        ).set(albums::is_faved.eq(now_faved))
            .execute(&conn)?;
        if updated == 1 {
            Ok(())
        }else{
            Err(DbError::Update(format!("Failed to set fav state on album {} to {}!", album_id, now_faved)))
        }
    }
}

fn _find_by_fav_status(pool: &DbPool, faved: bool, offset: i64, limit: i64) -> Result<Vec<Album>> {
    use crate::db_new::schema::albums::dsl::*;
    let conn = pool.get()?;
    let results = albums
        .filter(is_faved.eq(faved))
        .limit(limit)
        .offset(offset)
        .load::<Album>(&conn);
    Ok(results?)
}