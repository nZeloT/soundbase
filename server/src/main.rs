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
use std::sync::Arc;
use tokio::sync::RwLock;

use tonic::transport::Server;
use url::Url;
use crate::playback::local_player::LocalPlayer;
use crate::playback::PlaybackController;
use crate::playback::spotify_player::SpotifyPlayer;

use crate::services::definition::library_server::LibraryServer;
use crate::services::definition::tasks_server::TasksServer;
use crate::services::definition::spotify_auth_server::SpotifyAuthServer;
use crate::services::definition::playback_controls_server::PlaybackControlsServer;
use crate::services::library::LibraryService;
use crate::services::spotify_auth::SpotifyAuthService;
use crate::services::tasks::TasksService;
use crate::services::playback::PlaybackControlsService;
use crate::spotify::SpotifyApi;

mod model;
mod services;
mod tasks;
mod string_utils;
mod db_new;
mod spotify;
mod playback;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    dotenv::dotenv().ok();
    let url_env_val = dotenv::var("DATABASE_URL").expect("Failed to read ENV variable DATABASE_URL");
    let url = Url::parse(&*url_env_val).expect("Url is not valid!");
    let db_api = db_new::DbApi::new(url);

    let spotify = match SpotifyApi::new().await {
        Ok(s) => s,
        Err(e) => {
            println!("{:?}", e);
            return Ok(());
        }
    };

    let spot_user = dotenv::var("SPOT_USER").expect("Failed to read ENV variable SPOT_USER");
    let spot_pass = dotenv::var("SPOT_PASS").expect("Failed to read ENV variable SPOT_PASS");
    let spot_cache = ("./.spot_cache/system", "./.spot_cache/audio");
    let spotify_player = SpotifyPlayer::new(&*spot_user, &*spot_pass, spot_cache).await;
    let local_player = LocalPlayer::new();

    let mut playback_controller = PlaybackController::new(
        db_api.clone(),
        spotify_player,
        local_player
    )?;
    playback_controller.init().await;

    let library_service = LibraryService{
        db : db_api.clone()
    };

    let tasks_service = TasksService{
        db : db_api.clone(),
        spotify: spotify.clone()
    };

    let spotify_auth = SpotifyAuthService{
        spotify: spotify.clone()
    };

    let playback_service = PlaybackControlsService{
        playback : Arc::new(RwLock::new(playback_controller))
    };

    let env_ip_str = match dotenv::var("SERVER_IP") {
        Ok(given_ip) => given_ip,
        Err(_) => "192.168.2.111:3333".to_string()
    };
    let sock_addr: SocketAddr = env_ip_str.parse().unwrap();

    log::info!("Soundbase listening on => {}", env_ip_str);
    // warp::serve(api).run(sock_addr).await;
    Server::builder()
        .add_service(PlaybackControlsServer::new(playback_service))
        .add_service(LibraryServer::new(library_service))
        .add_service(TasksServer::new(tasks_service))
        .add_service(SpotifyAuthServer::new(spotify_auth))
        .serve(sock_addr)
        .await?;

    Ok(())
}