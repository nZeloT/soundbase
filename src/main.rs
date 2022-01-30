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

use std::net::SocketAddr;

use url::Url;
use warp::Filter;

use crate::spotify::SpotifyApi;

mod error;
mod model;
mod api_handler;
mod generated;
mod tasks;
mod string_utils;
mod db_new;
mod spotify;

type Result<T> = core::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, warp::Rejection>;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    dotenv::dotenv().ok();
    let url_env_val = dotenv::var("DATABASE_URL").expect("Failed to read ENV variable DATABASE_URL");
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
        spotify,
    ).recover(error::handle_rejection);


    let env_ip_str = match dotenv::var("SERVER_IP") {
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
        let prefix = warp::path!("api" / "v1" / ..);

        let list_ep = warp::path::end()
            .and(warp::get())
            .and(with_db(db.clone()))
            .and(warp::query::<model::RequestPage>());
        let single_ep = warp::path!(i32)
            .and(warp::get())
            .and(with_db(db.clone()));

        let tracks = list_ep.clone()
            .and_then(api_handler::library::load_tracks);
        let single_track = single_ep.clone()
            .and_then(api_handler::library::load_single_track);
        let tracks_api = warp::path!("tracks" / ..)
            .and(tracks.or(single_track));

        let albums = list_ep.clone()
            .and_then(api_handler::library::load_albums);
        let single_album = single_ep.clone()
            .and_then(api_handler::library::load_single_album);
        let album_api = warp::path!("albums" / ..)
            .and(albums.or(single_album));

        let artists = list_ep.clone()
            .and_then(api_handler::library::load_artists);
        let single_artist = single_ep
            .and_then(api_handler::library::load_single_artist);
        let artist_api = warp::path!("artists" / ..)
            .and(artists.or(single_artist));

        let library_api = warp::path!("library" / ..)
            .and(tracks_api.or(album_api).or(artist_api));

        let proposals = list_ep
            .and_then(api_handler::track_proposals::load_proposals);
        let prop_matches = warp::path!(i32 / "matches")
            .and(warp::get())
            .and(warp::query::<api_handler::track_proposals::MatchesQuery>())
            .and(with_db(db.clone()))
            .and(with_spotify(spotify.clone()))
            .and_then(api_handler::track_proposals::load_proposal_matches);
        let prop_confirm = warp::path!(i32 / "confirm" / String)
            .and(warp::get())
            .and(with_db(db.clone()))
            .and(with_spotify(spotify.clone()))
            .and_then(api_handler::track_proposals::confirm_proposal);
        let prop_discard = warp::path!(i32 / "discard")
            .and(warp::get())
            .and(with_db(db.clone()))
            .and_then(api_handler::track_proposals::discard_proposal);
        let track_proposal_api = warp::path!("track-proposals" / ..)
            .and(proposals
                .or(prop_matches)
                .or(prop_confirm)
                .or(prop_discard)
            );

        let album_of_week = warp::path!("albums-of-week")
            .and(warp::get())
            .and(with_db(db.clone()))
            .and_then(api_handler::tasks::fetch_albums_of_week);
        let charts = warp::path!("charts")
            .and(warp::get())
            .and(with_db(db.clone()))
            .and_then(api_handler::tasks::fetch_charts);
        let spot_import = warp::path!("import")
            .and(warp::get())
            .and(with_db(db.clone()))
            .and(with_spotify(spotify.clone()))
            .and_then(api_handler::tasks::import_from_spotify);
        let task_api = warp::path!("tasks" / ..)
            .and(album_of_week
                .or(charts)
                .or(spot_import)
            );

        let song_like_proto = warp::path!("song-like-proto")
            .and(warp::post())
            .and(with_db(db))
            .and(warp::body::bytes())
            .and_then(api_handler::song_like::handle_song_like_message);

        let heartbeat = warp::path!("heartbeat")
            .and(warp::get())
            .and_then(api_handler::heartbeat);

        spotify_auth_callback(spotify)
            .or(
                prefix.and(
                    heartbeat
                        .or(song_like_proto)
                        .or(task_api)
                        .or(library_api)
                        .or(track_proposal_api)
                )
            )
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

    fn with_spotify(spot: SpotifyApi) -> impl Filter<Extract=(SpotifyApi, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || spot.clone())
    }
}