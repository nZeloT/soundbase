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
use crate::db_new::FindById;
use crate::db_new::models::{Genre, NewGenre};
use crate::db_new::schema::*;

pub trait GenreDb: FindById<Genre> + Sync {
    fn new_genre(&self, name: &str) -> Result<Genre>;
    fn find_genre_by_name(&self, name: &str) -> Result<Genre>;
}

impl GenreDb for DbApi {
    fn new_genre(&self, name: &str) -> Result<Genre> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(genre::table)
            .values(&NewGenre {
                name
            })
            .get_result(&conn);
        Ok(result?)
    }

    fn find_genre_by_name(&self, genre_name: &str) -> Result<Genre> {
        use crate::db_new::schema::genre::dsl::*;
        let conn = self.0.get()?;
        let result = genre
            .filter(name.eq(genre_name))
            .first(&conn);
        Ok(result?)
    }
}

impl FindById<Genre> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<Option<Genre>> {
        use crate::db_new::schema::genre::dsl::*;
        let conn = self.0.get()?;
        let result = genre
            .find(id)
            .first(&conn)
            .optional();
        Ok(result?)
    }

    fn find_by_ids(&self, ids: Vec<i32>) -> Result<Vec<Genre>> {
        use diesel::dsl::any;
        let conn = self.0.get()?;
        let result = genre::table
            .filter(genre::genre_id.eq(any(ids)))
            .load::<Genre>(&conn);
        Ok(result?)
    }
}