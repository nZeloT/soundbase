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

use crate::db_new::{DbApi, DbError, Result};
use crate::db_new::FindById;
use crate::db_new::models::{AlbumOfWeek, NewAlbumOfWeek};
use crate::db_new::schema::*;

pub trait AlbumOfWeekDb : FindById<AlbumOfWeek> {
    fn new_album_of_week(&self, new_aow : NewAlbumOfWeek) -> Result<AlbumOfWeek>;
    fn find_by_source_and_week(&self, source : &str, year : i32, week : i32) -> Result<Option<AlbumOfWeek>>;
}

impl AlbumOfWeekDb for DbApi {
    fn new_album_of_week(&self, new_aow: NewAlbumOfWeek) -> Result<AlbumOfWeek> {
        let conn = self.0.get()?;
        let result = diesel::insert_into(albums_of_week::table)
            .values(&new_aow)
            .get_result(&conn);
        Ok(result?)
    }

    fn find_by_source_and_week(&self, source: &str, year: i32, week: i32) -> Result<Option<AlbumOfWeek>> {
        let conn = self.0.get()?;
        let result = albums_of_week::table
            .filter(albums_of_week::year.eq(year))
            .filter(albums_of_week::week.eq(week))
            .filter(albums_of_week::source_name.ilike(source))
            .first(&conn)
            .optional();
        Ok(result?)
    }
}

impl FindById<AlbumOfWeek> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<AlbumOfWeek> {
        use crate::db_new::schema::albums_of_week::dsl::*;
        let conn = self.0.get()?;
        let result = albums_of_week
            .find(id)
            .first(&conn);
        Ok(result?)
    }
}