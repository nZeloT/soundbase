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
use crate::db_new::models::{Genre, NewGenre};
use crate::db_new::schema::*;
use crate::db_new::{FindById, UpdateSingle};

pub trait GenreDb: FindById<Genre> + UpdateSingle<Genre> {
    fn new_genre(&self, name: &str) -> Result<Genre>;
    fn find_genre_by_name(&self, name: &str) -> Result<Genre>;
}

impl GenreDb for DbApi {
    fn new_genre(&self, name: &str) -> Result<Genre> {
        match self.0.get() {
            Ok(conn) => {
                let new_genre = NewGenre{
                    name
                };
                let result = diesel::insert_into(genre::table)
                    .values(&new_genre)
                    .get_result(&conn);
                match result {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn find_genre_by_name(&self, genre_name: &str) -> Result<Genre> {
        use crate::db_new::schema::genre::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = genre
                    .filter(name.eq(genre_name))
                    .first(&conn);
                match result {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl FindById<Genre> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<Genre> {
        use crate::db_new::schema::genre::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = genre
                    .find(id)
                    .first(&conn);
                match result {
                    Ok(value) => Ok(value),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl UpdateSingle<Genre> for DbApi {
    fn update(&self, to_update: &Genre) -> Result<()> {
        use crate::db_new::schema::genre::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::update(genre)
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