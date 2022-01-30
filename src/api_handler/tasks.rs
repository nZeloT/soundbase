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

use warp::Reply;
use crate::db_new::DbApi;
use crate::{SpotifyApi, tasks, WebResult};

pub async fn fetch_charts(db: DbApi) -> WebResult<impl Reply> {
    tasks::launch_fetch_charts(&db);
    Ok(format!(""))
}

pub async fn fetch_albums_of_week(db: DbApi) -> WebResult<impl Reply> {
    tasks::launch_fetch_albums_of_week(&db);
    Ok(format!(""))
}

pub async fn import_from_spotify(db : DbApi, spotify : SpotifyApi) -> WebResult<impl Reply> {
    tasks::launch_spotify_import(&db, &spotify);
    Ok(format!(""))
}