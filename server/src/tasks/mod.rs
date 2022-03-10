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

use thiserror::Error;

use crate::db_new::DbApi;
use crate::spotify::SpotifyApi;
use crate::tasks::spotify_import::SpotifyImporter;

mod aow_rock_antenne;
mod tow_rock_antenne;
mod spotify_import;

type Result<T> = std::result::Result<T, TasksError>;

pub fn launch_fetch_albums_of_week(db : &DbApi) {
    let api = db.clone();
    tokio::task::spawn(async move {
        if let Err(e) = aow_rock_antenne::fetch_new_rockantenne_album_of_week(api).await {
            println!("AOW Fetch for Rock Antenne raised an Error! => {:?}", e);
        }
    });
}

pub fn launch_fetch_charts(db : &DbApi) {
    let api = db.clone();
    tokio::task::spawn(async move {
        if let Err(e) = tow_rock_antenne::fetch_new_rockantenne_top20_of_week(api).await {
            println!("Charts Fetch for Rock Antenne raised an Error! => {:?}", e);
        }
    });
}

pub fn launch_spotify_import(db : &DbApi, spotify : &SpotifyApi) {
    let db = db.clone();
    let spotify = spotify.clone();
    tokio::task::spawn(async move {
        if let Err(e) = SpotifyImporter::new(db, spotify).do_import().await {
            println!("Error occured during spotify import! => {:?}", e);
        }
    });
}

fn get_selector(selector: &'static str) -> Result<scraper::Selector> {
    let sel = scraper::Selector::parse(selector);
    match sel {
        Ok(s) => Ok(s),
        Err(e) => Err(TasksError::Internal(format!("{:?}", e)))
    }
}

#[derive(Error, Debug)]
pub enum TasksError {
    #[error("internal error: {0}")]
    Internal(String),

    #[error("Reqwest error: {0}")]
    Requwest(#[from] reqwest::Error),

    #[error("Chrono parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),

    #[error("database error: {0}")]
    Database(#[from] super::db_new::DbError),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Int parse error: {0}")]
    NumberParse(#[from] std::num::ParseIntError),

    #[error("Utf8 parse error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("input/output error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Spotify api error: {0}")]
    SpotifyApi(#[from] crate::spotify::SpotifyApiError),
}