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

use std::sync::Arc;

use http::StatusCode;
use tokio::sync::RwLock;

use crate::db::DbPool;
use crate::error::{Result};
use crate::fetch;
use crate::model::song_like::SourceMetadataDetermination;

mod song_like_handler;

type Dissects = Arc<Vec<SourceMetadataDetermination>>;
type Spotify = Arc<RwLock<super::model::spotify::Spotify>>;

pub async fn heartbeat() -> Result<impl warp::Reply, std::convert::Infallible> {
    println!("Received a Heartbeat request.");
    println!();
    Ok(reply(String::from(""), StatusCode::OK))
}

pub async fn song_fav(body: bytes::Bytes, spotify: Spotify, dissects: Dissects) -> Result<impl warp::Reply, std::convert::Infallible> {
    let resp = song_like_handler::consume_like_message(spotify, &dissects, body.to_vec()).await;

    match resp {
        Ok(r) => Ok(reply(r, http::StatusCode::OK)),
        Err(e) => {
            println!("\tResponding with Error => {:?}", e.msg);
            Ok(reply(e.msg.as_bytes().to_vec(), e.http_code))
        }
    }
}

pub async fn fetch_charts(db: DbPool) -> Result<impl warp::Reply, std::convert::Infallible> {
    fetch::fetch_charts(&db);
    Ok(reply(String::from(""), http::StatusCode::OK))
}

pub async fn fetch_albums_of_week(db: DbPool) -> Result<impl warp::Reply, std::convert::Infallible> {
    fetch::fetch_albums_of_week(&db);
    Ok(reply(String::from(""), StatusCode::OK))
}

pub async fn spotify_start_auth(wrapper: Spotify) -> Result<impl warp::Reply, std::convert::Infallible> {
    let spotify = wrapper.read().await;
    let uri = spotify.get_authorization_url().await.to_string();
    Ok(reply(uri, http::StatusCode::OK))
}

pub async fn spotify_auth_callback(wrapper: Spotify, query: String) -> Result<impl warp::Reply, std::convert::Infallible> {
    let mut spotify = wrapper.write().await;
    match spotify.finish_initialization_with_code(query.as_str()).await {
        Ok(_) => {
            Ok(reply("Successful Authentication.".to_owned(), StatusCode::OK))
        }
        Err(e) => {
            println!("\tResponding with Error => {:?}", e);
            Ok(reply(e.msg, e.http_code))
        }
    }
}


fn reply<T: warp::Reply>(r: T, status: StatusCode) -> impl warp::Reply {
    warp::reply::with_status(r, status)
}


