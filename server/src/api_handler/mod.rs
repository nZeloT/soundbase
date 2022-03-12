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

use std::collections::HashMap;

use warp::reply::Reply;

use crate::{SpotifyApi, WebResult};
use crate::error::Error;

pub mod track_proposals;
pub mod song_like;

pub async fn heartbeat() -> WebResult<impl Reply> {
    println!("Received a Heartbeat request.");
    println!();
    Ok(format!(""))
}