/*
 * Copyright 2021 nzelot<leontsteiner@gmail.com>
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

#[derive(Debug)]
pub struct DbError(String);
impl DbError {
    pub fn new(msg: &str) -> Self {
        DbError(msg.to_string())
    }
}

impl From<DbError> for crate::error::SoundbaseError {
    fn from(e: DbError) -> Self {
        crate::error::SoundbaseError {
            msg: e.0.to_string(),
            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        let msg = format!("{:?}", e);
        DbError(msg)
    }
}

impl From<chrono::ParseError> for DbError {
    fn from(e: chrono::ParseError) -> Self {
        DbError(e.to_string())
    }
}

impl From<r2d2::Error> for DbError {
    fn from(e: r2d2::Error) -> Self {
        DbError(e.to_string())
    }
}