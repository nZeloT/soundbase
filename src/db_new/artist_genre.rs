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
use crate::db_new::models::{Artist, ArtistGenre, Genre, NewArtistGenre};
use crate::db_new::schema::*;
use crate::db_new::UpdateSingle;

pub trait ArtistGenreDb : UpdateSingle<ArtistGenre>
{
    fn new_artist_genre(&self, artist_id : i32, genre_id : i32) -> Result<ArtistGenre>;
    fn load_artists_for_genre(&self, genre : &Genre, offset : i64, limit : i64) -> Result<Vec<Artist>>;
    fn load_genres_for_artist(&self, artist : &Artist, offset : i64, limit : i64) -> Result<Vec<Genre>>;
}

impl ArtistGenreDb for DbApi {
    fn new_artist_genre(&self, artist_id: i32, genre_id: i32) -> Result<ArtistGenre> {
        match self.0.get() {
            Ok(conn) => {
                let new_artist_genre = NewArtistGenre{
                    artist_id,
                    genre_id
                };
                let result = diesel::insert_into(artist_genre::table)
                    .values(&new_artist_genre)
                    .get_result(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn load_artists_for_genre(&self, genre: &Genre, offset: i64, limit: i64) -> Result<Vec<Artist>> {
        use crate::db_new::schema::artist_genre::dsl::*;
        use diesel::dsl::any;

        match self.0.get() {
            Ok(conn) => {
                let genre_artist_ids = ArtistGenre::belonging_to(genre).select(artist_id);
                let result = artists::table
                    .filter(artists::artist_id.eq(any(genre_artist_ids)))
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

    fn load_genres_for_artist(&self, artist: &Artist, offset: i64, limit: i64) -> Result<Vec<Genre>> {
        use crate::db_new::schema::artist_genre::dsl::*;
        use diesel::dsl::any;

        match self.0.get() {
            Ok(conn) => {
                let genre_artist_ids = ArtistGenre::belonging_to(artist).select(genre_id);
                let result = genre::table
                    .filter(genre::genre_id.eq(any(genre_artist_ids)))
                    .limit(limit)
                    .offset(offset)
                    .load::<Genre>(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}


impl UpdateSingle<ArtistGenre> for DbApi {
    fn update(&self, to_update: &ArtistGenre) -> Result<()> {
        use crate::db_new::schema::artist_genre::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::update(artist_genre)
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