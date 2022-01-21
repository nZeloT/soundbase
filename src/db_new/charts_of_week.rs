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
use crate::db_new::db_error::DbError;
use crate::db_new::models::{ChartsOfWeekEntry, NewChartsOfWeek};
use crate::db_new::schema::*;

pub trait ChartsOfWeekDb: FindById<ChartsOfWeekEntry> {
    fn new_charts_of_week_entry(&self, new_charts_entry: NewChartsOfWeek) -> Result<ChartsOfWeekEntry>;
    fn find_by_source_and_week(&self, source: &str, year: i32, week: i32) -> Result<Option<Vec<ChartsOfWeekEntry>>>;
}

impl ChartsOfWeekDb for DbApi {
    fn new_charts_of_week_entry(&self, new_charts_entry: NewChartsOfWeek) -> Result<ChartsOfWeekEntry> {
        match self.0.get() {
            Ok(conn) => {
                let result = diesel::insert_into(charts_of_week::table)
                    .values(&new_charts_entry)
                    .get_result(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }

    fn find_by_source_and_week(&self, source: &str, year: i32, week: i32) -> Result<Option<Vec<ChartsOfWeekEntry>>> {
        match self.0.get() {
            Ok(conn) => {
                let result = charts_of_week::table
                    .filter(charts_of_week::year.eq(year))
                    .filter(charts_of_week::calendar_week.eq(week))
                    .filter(charts_of_week::source_name.ilike(source))
                    .load::<ChartsOfWeekEntry>(&conn)
                    .optional();
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl FindById<ChartsOfWeekEntry> for DbApi {
    fn find_by_id(&self, id: i32) -> Result<ChartsOfWeekEntry> {
        use crate::db_new::schema::charts_of_week::dsl::*;
        match self.0.get() {
            Ok(conn) => {
                let result = charts_of_week
                    .find(id)
                    .first(&conn);
                match result {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DbError::from(e))
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}