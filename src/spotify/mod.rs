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

use std::default::Default;
use std::str::FromStr;
use std::sync::Arc;

use rspotify::{AuthCodeSpotify, Config, Credentials, OAuth, scopes};
use rspotify::clients::{BaseClient, OAuthClient};
use rspotify::model::{AlbumId, ArtistId, FullAlbum, FullArtist, FullTrack, SearchResult};
use rspotify::model::{Market, SearchType, TrackId};
use tokio::sync::RwLock;

use crate::error::Error;
use crate::model::RequestPage;
use crate::Result;

pub mod db_utils;

#[derive(Clone)]
pub struct SpotifyApi(Arc<RwLock<AuthCodeSpotify>>);

impl SpotifyApi {
    pub async fn new() -> Result<Self> {
        Ok(SpotifyApi(Arc::new(RwLock::new(SpotifyApi::_init().await?))))
    }

    pub async fn search(&self, query: &str, page: RequestPage) -> Result<Vec<FullTrack>> {
        let client = self.0.read().await;
        match client.search(query,
                            &SearchType::Track,
                            Some(&Market::FromToken),
                            None,
                            Some(page.limit() as u32),
                            Some(page.offset() as u32)).await? {
            SearchResult::Tracks(page) => {
                Ok(page.items)
            }
            _ => Err(Error::InternalError("Expected Tracks but didn't get!".to_string()))
        }
    }

    pub async fn get_track(&self, id : &TrackId) -> Result<FullTrack> {
        let client = self.0.read().await;
        Ok(client.track(id).await?)
    }

    pub async fn get_track_from(&self, id: &str) -> Result<FullTrack> {
        let track_id = TrackId::from_str(id)?;
        self.get_track(&track_id).await
    }

    pub async fn get_album(&self, album_id: &AlbumId) -> Result<FullAlbum> {
        let client = self.0.read().await;
        Ok(client.album(album_id).await?)
    }

    pub async fn get_artists(&self, artist_ids: &[ArtistId]) -> Result<Vec<FullArtist>> {
        let client = self.0.read().await;
        Ok(client.artists(artist_ids).await?)
    }

    pub async fn get_saved_artists(&self, after : Option<&str>, limit : Option<u32>) -> Result<(i32, Vec<FullArtist>)> {
        let client = self.0.read().await;
        let page = client.current_user_followed_artists(after, limit).await?;
        Ok((page.total.unwrap() as i32, page.items))
    }

    pub async fn get_saved_albums(&self, page : &RequestPage) -> Result<(i32, Vec<FullAlbum>)> {
        let client = self.0.read().await;
        let page = client.current_user_saved_albums_manual(
            Some(&Market::FromToken),
            Some(page.limit() as u32),
            Some(page.offset() as u32)
        ).await?;
        Ok((page.total as i32, page.items.iter().map(|a|a.album.clone()).collect::<Vec<FullAlbum>>()))
    }

    pub async fn get_saved_tracks(&self, page : &RequestPage) -> Result<(i32, Vec<FullTrack>)> {
        let client = self.0.read().await;
        let page = client.current_user_saved_tracks_manual(
            Some(&Market::FromToken),
            Some(page.limit() as u32),
            Some(page.offset() as u32)
        ).await?;
        Ok((page.total as i32, page.items.iter().map(|t|t.track.clone()).collect::<Vec<FullTrack>>()))
    }

    pub async fn save_track(&self, id: &str) -> Result<()>
    {
        let client = self.0.read().await;
        let track_id = TrackId::from_str(id)?;
        let _ = client.current_user_saved_tracks_add(vec![&track_id]).await?;
        Ok(())
    }

    pub async fn remove_saved_track(&self, id: &str) -> Result<()>
    {
        let client = self.0.read().await;
        let track_id = TrackId::from_str(id)?;
        let _ = client.current_user_saved_tracks_delete(vec![&track_id]).await?;
        Ok(())
    }

    pub async fn finish_initialization_with_code(&self, code: &str) -> Result<()> {
        let _ = self.0.write().await.request_token(&code).await?;
        Ok(())
    }

    async fn _init() -> Result<AuthCodeSpotify> {
        let (redir, id, secret) = SpotifyApi::_get_env_vars()?;
        let config = Config {
            token_cached: true,
            token_refreshing: true,
            ..Default::default()
        };

        if std::path::Path::new(&rspotify::DEFAULT_CACHE_PATH).exists() {}

        let oauth = OAuth {
            scopes: scopes!("user-library-read", "user-library-modify", "user-follow-modify", "user-follow-read"),
            redirect_uri: redir,
            ..Default::default()
        };

        let creds = Credentials::new(&*id, &*secret);
        let mut client = AuthCodeSpotify::with_config(creds, oauth, config);
        match client.read_token_cache(true).await {
            Ok(opt) => {
                match opt {
                    Some(token) => {
                        *client.get_token().lock().await.unwrap() = Some(token);
                        client.refresh_token().await?;
                        println!("Successfully refreshed spotify access token");
                    }
                    None => {
                        println!("Infeasible token, reauthenticate!");
                        println!("Authenticate with spotify at {}", client.get_authorize_url(false)?);
                    }
                }
            }
            Err(e) => {
                println!("Couldn't read token cache! reauthenticate! => {:?}", e);
                println!("Authenticate with spotify at {}", client.get_authorize_url(false)?);
            }
        }

        Ok(client)
    }

    fn _get_env_vars() -> Result<(String, String, String)> {
        let redir_url = match dotenv::var("REDIRECT_URI") {
            Ok(uri) => uri,
            Err(_) => return Err(Error::InternalError(format!("Failed to read REDIRECT_URI!")))
        };

        let client_id = match dotenv::var("CLIENT_ID") {
            Ok(client_id) => client_id,
            Err(_) => return Err(Error::InternalError(format!("Failed to read CLIENT_ID!")))
        };

        let client_sec = match dotenv::var("CLIENT_SECRET") {
            Ok(secret) => secret,
            Err(_) => return Err(Error::InternalError(format!("Failed to read CLIENT_SECRET!")))
        };

        Ok((redir_url, client_id, client_sec))
    }
}