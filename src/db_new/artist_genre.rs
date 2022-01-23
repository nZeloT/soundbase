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
use crate::db_new::models::{Artist, ArtistGenre, Genre, NewArtistGenre};
use crate::db_new::schema::*;

pub trait ArtistGenreDb: Sync {
    fn new_artist_genre(&self, artist_id: i32, genre_id: i32) -> Result<ArtistGenre>;
    fn load_artists_for_genre(&self, genre: &Genre, offset: i64, limit: i64) -> Result<Vec<Artist>>;
    fn load_genres_for_artist(&self, artist: &Artist, offset: i64, limit: i64) -> Result<Vec<Genre>>;
}

impl ArtistGenreDb for DbApi {
    fn new_artist_genre(&self, artist_id: i32, genre_id: i32) -> Result<ArtistGenre> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(artist_genre::table)
            .values(&NewArtistGenre {
                artist_id,
                genre_id,
            })
            .get_result(&conn);
        Ok(result?)
    }

    fn load_artists_for_genre(&self, genre: &Genre, offset: i64, limit: i64) -> Result<Vec<Artist>> {
        use crate::db_new::schema::artist_genre::dsl::*;
        use diesel::dsl::any;

        let conn = self.0.get()?;
        let genre_artist_ids = ArtistGenre::belonging_to(genre).select(artist_id);
        let result = artists::table
            .filter(artists::artist_id.eq(any(genre_artist_ids)))
            .limit(limit)
            .offset(offset)
            .load::<Artist>(&conn);
        Ok(result?)
    }

    fn load_genres_for_artist(&self, artist: &Artist, offset: i64, limit: i64) -> Result<Vec<Genre>> {
        use crate::db_new::schema::artist_genre::dsl::*;
        use diesel::dsl::any;

        let conn = self.0.get()?;
        let genre_artist_ids = ArtistGenre::belonging_to(artist).select(genre_id);
        let result = genre::table
            .filter(genre::genre_id.eq(any(genre_artist_ids)))
            .limit(limit)
            .offset(offset)
            .load::<Genre>(&conn);
        Ok(result?)
    }
}