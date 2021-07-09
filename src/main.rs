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

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

mod error;
pub mod model;
pub mod db;
pub mod handlers;
pub mod generated;

#[tokio::main]
async fn main() {
    let db = db::initialize_db().expect("Failed to create DB!");

    let metadata_dissect = model::song_like::SourceMetadataDeterminationConfig::load_from_file("./config.json");
    println!("Read the following Metadata dissects:");
    println!("\t{:?}", metadata_dissect);
    println!();

    let mut spotify = crate::model::spotify::Spotify::new().unwrap();
    match spotify.finish_initialization_from_cache().await {
        Ok(_) => println!("Spotify access enabled."),
        Err(e) =>
            println!("Couldn't load spotify access token from cache (Error: {:?}). Consider authenticating by calling /spotify/start_auth.", e)
    }

    let api = filters::endpoints(db, Arc::new(metadata_dissect.sources), Arc::new(RwLock::new(spotify)));


    let env_ip_str = match std::env::var("SERVER_IP") {
        Ok(given_ip) => given_ip,
        Err(_) => "192.168.2.111:3333".to_string()
    };
    let sock_addr: SocketAddr = env_ip_str.parse().unwrap();


    println!("Soundbase listening on => {}", env_ip_str);
    warp::serve(api).run(sock_addr).await;
}

mod filters {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use warp::Filter;

    use super::handlers;

    type DB = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
    type Dissects = Arc<Vec<super::model::song_like::SourceMetadataDetermination>>;
    type Spotify = Arc<RwLock<super::model::spotify::Spotify>>;

    pub fn endpoints(
        db: DB,
        dissects: Dissects,
        spotify: Spotify,
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        song_fav_heartbeat()
            .or(song_fav( spotify.clone(), dissects))
            .or(fetch_tow(db.clone()))
            .or(fetch_aow(db))
            .or(spotify_start_authorization(spotify.clone()))
            .or(spotify_auth_callback(spotify))
    }

    pub fn song_fav_heartbeat() -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("song_fav" / "heartbeat")
            .and(warp::get())
            .and_then(handlers::heartbeat)
    }

    pub fn song_fav(
        spotify: Spotify,
        dissects: Dissects,
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("song_fav")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_spotify(spotify))
            .and(with_dissects(dissects))
            .and_then(handlers::song_fav)
    }

    pub fn fetch_tow(
        db: DB
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("fetch" / "Top20OfWeek")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::fetch_top_20_of_week)
    }

    pub fn fetch_aow(
        db: DB
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("fetch" / "AlbumOfWeek")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::fetch_album_of_week)
    }

    pub fn spotify_start_authorization(
        spotify: Spotify
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("spotify" / "start_auth")
            .and(warp::get())
            .and(with_spotify(spotify))
            .and_then(handlers::spotify_start_auth)
    }

    pub fn spotify_auth_callback(
        spotify: Spotify
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("spotify" / "auth_callback")
            .and(warp::get())
            .and(with_spotify(spotify))
            .and(warp::query::raw())
            .and_then(handlers::spotify_auth_callback)
    }

    fn with_db(db: DB) -> impl Filter<Extract=(DB, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn with_dissects(dissects: Dissects) -> impl Filter<Extract=(Dissects, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || dissects.clone())
    }

    fn with_spotify(spot: Spotify) -> impl Filter<Extract=(Spotify, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || spot.clone())
    }
}