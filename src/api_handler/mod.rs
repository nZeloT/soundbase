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
use std::fmt::Debug;

use http::StatusCode;
use serde::{Deserialize, Serialize};
use warp::reply::Reply;

use crate::db_new::db_error::DbError;
use crate::error::{Result, SoundbaseError};
use crate::SpotifyApi;

pub mod tasks;
pub mod track_proposals;
pub mod song_like;

#[derive(Serialize, Deserialize)]
struct ApiError {
    msg : String
}

impl From<SoundbaseError> for ApiError {
    fn from(e: SoundbaseError) -> Self {
        Self{msg: e.msg}
    }
}

impl From<DbError> for ApiError {
    fn from(e: DbError) -> Self {
        Self{msg : format!("{:?}", e)}
    }
}

pub async fn heartbeat() -> Result<impl warp::Reply, std::convert::Infallible> {
    println!("Received a Heartbeat request.");
    println!();
    Ok(reply(String::from(""), StatusCode::OK))
}

// pub async fn spotify_start_auth(api: SpotifyApi) -> Result<impl warp::Reply, std::convert::Infallible> {
//     let uri = spotify.get_authorization_url().await.to_string();
//     Ok(reply(uri, http::StatusCode::OK))
// }

pub async fn spotify_auth_callback(api: SpotifyApi, params: HashMap<String, String>) -> Result<impl warp::Reply, std::convert::Infallible> {
    match params.get("code") {
        Some(code) => {
            match api.finish_initialization_with_code(code).await {
                Ok(_) => Ok(reply("Successful Authentication.".to_owned(), StatusCode::OK)),
                Err(e) => {
                    println!("\tResponding with Error => {:?}", e);
                    Ok(reply(e.msg, e.http_code))
                }
            }
        }
        None => {
            Ok(reply("Failed to read parameter 'code' from query parameters!".to_owned(), StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}


fn reply<T: warp::Reply>(r: T, status: StatusCode) -> impl warp::Reply {
    warp::reply::with_status(r, status)
}

fn reply_json<T>(r : &T, status : StatusCode) -> impl warp::Reply
where T : Serialize {
    reply(warp::reply::json(r), status)
}

fn handle_error<E>(message : &str, e : E) -> warp::reply::Response
where E : Into<ApiError> + Debug {
    println!("{} => {:?}", message, e);
    let ret = e.into();
    reply_json(&ret, StatusCode::INTERNAL_SERVER_ERROR).into_response()
}