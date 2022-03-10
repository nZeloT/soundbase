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
use crate::db_new::models::{Artist, NewTrackArtists, Track, TrackArtists};
use crate::db_new::schema::*;
use crate::model::RequestPage;

pub trait TrackArtistsDb {
    fn new_track_artist(&self, track_id : i32, artist_id : i32) -> Result<TrackArtists>;
    fn new_track_artist_if_missing(&self, track_id : i32, artist_id : i32) -> Result<TrackArtists>;
    fn load_track_for_artist(&self, artist: &Artist, page : RequestPage) -> Result<Vec<Track>>;
    fn load_artists_for_track(&self, track : &Track) -> Result<Vec<Artist>>;
    fn load_artist_ids_for_tracks(&self, tracks : &[Track]) -> Result<HashMap<i32, Vec<i32>>>;
}

impl TrackArtistsDb for DbApi {
    fn new_track_artist(&self, track_id: i32, artist_id: i32) -> Result<TrackArtists> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(track_artist::table)
            .values(&NewTrackArtists{
                track_id,
                artist_id
            })
            .get_result(&conn);
        Ok(result?)
    }

    fn new_track_artist_if_missing(&self, track_id: i32, artist_id: i32) -> Result<TrackArtists> {
        let conn = self.0.get()?;
        let option = track_artist::table
                    .filter(track_artist::track_id.eq(track_id))
                    .filter(track_artist::artist_id.eq(artist_id))
                    .first(&conn)
                    .optional()?;

        match option {
            Some(link) => Ok(link),
            None => self.new_track_artist(track_id, artist_id)
        }
    }
    
    fn load_track_for_artist(&self, artist: &Artist, page: RequestPage) -> Result<Vec<Track>> {
        use crate::db_new::schema::track_artist::dsl::*;
        use diesel::dsl::any;

        let conn = self.0.get()?;
        let track_ids = TrackArtists::belonging_to(artist).select(track_id);
        let result = tracks::table
            .filter(tracks::track_id.eq(any(track_ids)))
            .limit(page.limit())
            .offset(page.offset())
            .load::<Track>(&conn);
        Ok(result?)
    }

    fn load_artists_for_track(&self, track: &Track) -> Result<Vec<Artist>> {
        use crate::db_new::schema::track_artist::dsl::*;
        use diesel::dsl::any;

        let conn = self.0.get()?;
        let artist_ids = TrackArtists::belonging_to(track).select(artist_id);
        let result = artists::table
            .filter(artists::artist_id.eq(any(artist_ids)))
            .load::<Artist>(&conn);
        Ok(result?)
    }

    fn load_artist_ids_for_tracks(&self, tracks: &[Track]) -> Result<HashMap<i32, Vec<i32>>> {
        use diesel::dsl::any;
        let track_ids = tracks.iter().map(|t| t.track_id).unique().collect_vec();
        let conn = self.0.get()?;
        let track_artists = track_artist::table
            .filter(track_artist::track_id.eq(any(track_ids)))
            .load::<TrackArtists>(&conn)?;

        let mut map : HashMap<i32, Vec<i32>> = HashMap::new();
        for track_artist in &track_artists {
            match map.get_mut(&track_artist.track_id) {
                Some(artists) => artists.insert(0, track_artist.artist_id),
                None => {
                    map.insert(track_artist.track_id, vec![track_artist.artist_id]);
                }
            }
        }

        Ok(map)
    }
}