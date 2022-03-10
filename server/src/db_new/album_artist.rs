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

use std::collections::HashMap;
use diesel::prelude::*;
use itertools::Itertools;

use crate::db_new::{DbApi, Result};
use crate::db_new::models::{Album, Artist, AlbumArtists, NewAlbumArtists};
use crate::db_new::schema::*;

pub trait AlbumArtistsDb : Sync {
    fn new_album_artist(&self, artist_id: i32, album_id: i32) -> Result<AlbumArtists>;
    fn new_album_artist_if_missing(&self, artist_id: i32, album_id: i32) -> Result<AlbumArtists>;
    fn load_albums_for_artist(&self, artist: &Artist) -> Result<Vec<Album>>;
    fn load_artists_for_album(&self, album: &Album) -> Result<Vec<Artist>>;
    fn load_artist_ids_for_albums(&self, albums : &[Album]) -> Result<HashMap<i32, Vec<i32>>>;
}

impl AlbumArtistsDb for DbApi {
    fn new_album_artist(&self, artist_id: i32, album_id: i32) -> Result<AlbumArtists> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(album_artists::table)
            .values(&NewAlbumArtists {
                album_id,
                artist_id,
            })
            .get_result(&conn);
        Ok(result?)
    }

    fn new_album_artist_if_missing(&self, artist_id: i32, album_id: i32) -> Result<AlbumArtists> {
        let conn = self.0.get()?;
        let result = album_artists::table
            .filter(album_artists::artist_id.eq(artist_id))
            .filter(album_artists::album_id.eq(album_id))
            .first(&conn)
            .optional()?;

        match result {
            Some(link) => Ok(link),
            None => self.new_album_artist(artist_id, album_id)
        }
    }

    fn load_albums_for_artist(&self, artist: &Artist) -> Result<Vec<Album>> {
        use crate::db_new::schema::album_artists::dsl::*;
        use diesel::dsl::any;

        let conn = self.0.get()?;
        let album_ids = AlbumArtists::belonging_to(artist).select(album_id);
        let result = albums::table
            .filter(albums::album_id.eq(any(album_ids)))
            .load::<Album>(&conn);
        Ok(result?)
    }

    fn load_artists_for_album(&self, album: &Album) -> Result<Vec<Artist>> {
        use crate::db_new::schema::album_artists::dsl::*;
        use diesel::dsl::any;

        let conn = self.0.get()?;
        let artist_ids = AlbumArtists::belonging_to(album).select(artist_id);
        let result = artists::table
            .filter(artists::artist_id.eq(any(artist_ids)))
            .load::<Artist>(&conn);
        Ok(result?)
    }

    fn load_artist_ids_for_albums(&self, albums: &[Album]) -> Result<HashMap<i32, Vec<i32>>> {
        use diesel::dsl::any;
        let album_ids = albums.iter().map(|a| a.album_id).unique().collect_vec();
        let conn = self.0.get()?;
        let album_artists = track_artist::table
            .filter(track_artist::track_id.eq(any(album_ids)))
            .load::<AlbumArtists>(&conn)?;

        let mut map : HashMap<i32, Vec<i32>> = HashMap::new();
        for album_artist in &album_artists {
            match map.get_mut(&album_artist.album_id) {
                Some(artists) => artists.insert(0, album_artist.artist_id),
                None => {
                    map.insert(album_artist.album_id, vec![album_artist.artist_id]);
                }
            }
        }

        Ok(map)
    }
}