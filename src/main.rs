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

#[macro_use]
extern crate diesel;

use std::env;
use std::net::SocketAddr;
use url::Url;

use crate::spotify::SpotifyApi;

mod error;
mod model;
mod api_handler;
mod generated;
mod tasks;
mod string_utils;
mod db_new;
mod spotify;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let url_env_val = env::var("DATABASE_URL").expect("Failed to read ENV variable DATABASE_URL");
    let url = Url::parse(&*url_env_val).expect("Url is not valid!");
    let db_api = db_new::DbApi::new(url);

    let spotify = match SpotifyApi::new().await {
        Ok(s) => s,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    let api = filters::endpoints(
        db_api,
        spotify
    );


    let env_ip_str = match std::env::var("SERVER_IP") {
        Ok(given_ip) => given_ip,
        Err(_) => "192.168.2.111:3333".to_string()
    };
    let sock_addr: SocketAddr = env_ip_str.parse().unwrap();


    println!("Soundbase listening on => {}", env_ip_str);
    warp::serve(api).run(sock_addr).await;
}

mod filters {
    use std::collections::HashMap;

    use warp::Filter;

    use crate::db_new::DbApi;
    use crate::model;
    use crate::spotify::SpotifyApi;

    use super::api_handler;

    pub fn endpoints(
        db: DbApi,
        // dissects: Dissects,
        spotify: SpotifyApi,
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        song_fav_heartbeat()
            .or(song_fav( db.clone()))
            .or(task_api(db.clone()))
            .or(track_proposal_api(db, spotify.clone()))
            // .or(spotify_start_authorization(spotify.clone()))
            .or(spotify_auth_callback(spotify))
    }

    fn song_fav_heartbeat() -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("song_fav" / "heartbeat")
            .and(warp::get())
            .and_then(api_handler::heartbeat)
    }

    fn song_fav(
        db : DbApi
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("song_fav")
            .and(warp::post())
            .and(with_db(db.clone()))
            .and(warp::body::bytes())
            .and_then(api_handler::song_like::handle_song_like_message)
    }

    fn task_api(
        db : DbApi
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        fetch_aow(db.clone())
            // .or(fetch_tow(db))
    }

    fn track_proposal_api(
        db : DbApi,
        spotify : SpotifyApi
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        track_proposal_load(db.clone())
            .or(track_proposal_confirm(db.clone(), spotify.clone()))
            .or(track_proposal_discard(db.clone()))
            .or(track_proposal_matches(db, spotify))
    }

    fn spotify_auth_callback(
        spotify: SpotifyApi
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("spotify" / "auth_callback")
            .and(warp::get())
            .and(with_spotify(spotify))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(api_handler::spotify_auth_callback)
    }

    fn fetch_tow(
        db: DbApi
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("fetch" / "Charts")
            .and(warp::get())
            .and(with_db(db))
            .and_then(api_handler::tasks::fetch_charts)
    }

    fn fetch_aow(
        db: DbApi
    ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("fetch" / "AlbumsOfWeek")
            .and(warp::get())
            .and(with_db(db))
            .and_then(api_handler::tasks::fetch_albums_of_week)
    }

    fn track_proposal_load(db : DbApi) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "track-proposal" )
            .and(warp::get())
            .and(with_db(db))
            .and(warp::query::<model::Page>())
            .and_then(api_handler::track_proposals::load_proposals)
    }

    fn track_proposal_confirm(db : DbApi, spotify : SpotifyApi) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "track-proposal" / i32 / "confirm" / String)
            .and(warp::get())
            .and(with_db(db))
            .and(with_spotify(spotify))
            .and_then(api_handler::track_proposals::confirm_proposal)
    }

    fn track_proposal_discard(db : DbApi) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "track-proposal" / i32 / "discard")
            .and(warp::get())
            .and(with_db(db))
            .and_then(api_handler::track_proposals::discard_proposal)
    }

    fn track_proposal_matches(db : DbApi, spotify : SpotifyApi) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "track-proposal" / i32 / "matches")
            .and(warp::get())
            .and(warp::query::<api_handler::track_proposals::MatchesQuery>())
            .and(with_db(db))
            .and(with_spotify(spotify))
            .and_then(api_handler::track_proposals::load_proposal_matches)
    }

    // pub fn spotify_start_authorization(
    //     spotify: SpotifyApi
    // ) -> impl warp::Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    //     warp::path!("spotify" / "start_auth")
    //         .and(warp::get())
    //         .and(with_spotify(spotify))
    //         .and_then(handlers::spotify_start_auth)
    // }



    fn with_db(db: DbApi) -> impl Filter<Extract=(DbApi, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
//
//     fn with_dissects(dissects: Dissects) -> impl Filter<Extract=(Dissects, ), Error=std::convert::Infallible> + Clone {
//         warp::any().map(move || dissects.clone())
//     }
//
    fn with_spotify(spot: SpotifyApi) -> impl Filter<Extract=(SpotifyApi, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || spot.clone())
    }
}