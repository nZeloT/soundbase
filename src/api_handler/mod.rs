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

use http::StatusCode;
use serde::{Serialize};
use warp::reply::Reply;

use crate::{SpotifyApi, WebResult};
use crate::error::Error;

pub mod tasks;
pub mod track_proposals;
pub mod song_like;

pub async fn heartbeat() -> WebResult<impl Reply> {
    println!("Received a Heartbeat request.");
    println!();
    Ok(format!(""))
}

// pub async fn spotify_start_auth(api: SpotifyApi) -> Result<impl warp::Reply, std::convert::Infallible> {
//     let uri = spotify.get_authorization_url().await.to_string();
//     Ok(reply(uri, http::StatusCode::OK))
// }

pub async fn spotify_auth_callback(api: SpotifyApi, params: HashMap<String, String>) -> WebResult<impl Reply> {
    match params.get("code") {
        Some(code) => {
            match api.finish_initialization_with_code(code).await {
                Ok(_) => Ok(format!("Successful Spotify Authentication")),
                Err(e) => Err(warp::reject::custom(e))
            }
        }
        None => Err(warp::reject::custom(Error::RequestError(format!("Couldn't parse query parameter 'code'!"))))
    }
}


fn reply<T: warp::Reply>(r: T, status: StatusCode) -> impl warp::Reply {
    warp::reply::with_status(r, status)
}

fn reply_json<T>(r : &T, status : StatusCode) -> impl warp::Reply
where T : Serialize {
    reply(warp::reply::json(r), status)
}