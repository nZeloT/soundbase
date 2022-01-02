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

use tokio::task;
use crate::error::{Result, SoundbaseError};
use crate::db::{DbPool};

mod aow_rock_antenne;
mod tow_rock_antenne;

pub fn fetch_albums_of_week(db : &DbPool) {
    let pool = db.clone();
    tokio::task::spawn(async move {
        aow_rock_antenne::fetch_new_rockantenne_album_of_week(pool).await;
    });
}

pub fn fetch_charts(db : &DbPool) {
    let pool = db.clone();
    tokio::task::spawn(async move {
        tow_rock_antenne::fetch_new_rockantenne_top20_of_week(pool).await;
    });
}

fn get_selector(selector: &'static str) -> Result<scraper::Selector> {
    let sel = scraper::Selector::parse(selector);
    match sel {
        Ok(s) => Ok(s),
        Err(e) => {
            Err(SoundbaseError {
                http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!("{:?}", e),
            })
        }
    }
}