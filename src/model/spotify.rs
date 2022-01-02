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

use aspotify::Scope;
use serde::{Deserialize, Serialize};

use crate::db::album::Album;
use crate::db::artist::Artist;
use crate::db::FollowForeignReference;
use crate::db::song::Song;
use crate::error;
use crate::error::SoundbaseError;
use crate::model::song_like::RawSong;

const NECESSARY_SCOPES: [Scope; 2] = [
    Scope::UserLibraryRead,
    Scope::UserLibraryModify,
];

pub struct SpotifySong {
    track_id: String,
    pub in_library: bool,
}

#[derive(Debug)]
pub struct Spotify {
    state: String,
    auth_url: String,
    redirect_url: String,
    client: aspotify::Client,
    is_initialized: bool,
}

#[derive(Debug,Serialize)]
#[serde(tag = "grant_type", rename = "refresh_token")]
struct ForceTokenRefresh {
    refresh_token: String
}

#[derive(Debug, Deserialize, Serialize)]
struct AccessToken {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>
}

impl Spotify {
    pub fn new() -> Result<Self, error::SoundbaseError> {
        let creds = match aspotify::ClientCredentials::from_env() {
            Ok(c) => c,
            Err(e) => return Err(SoundbaseError {
                msg: e.to_string(),
                http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            })
        };

        let redirect_uri = match std::env::var("REDIRECT_URI") {
            Ok(uri) => uri,
            Err(e) => return Err(SoundbaseError {
                msg: e.to_string(),
                http_code: http::StatusCode::INTERNAL_SERVER_ERROR
            })
        };

        let (url, state) = aspotify::authorization_url(
            &creds.id,
            NECESSARY_SCOPES.iter().copied(),
            false,
            redirect_uri.as_str(),
        );

        Ok(Spotify {
            state,
            redirect_url: redirect_uri,
            auth_url: url,
            client: aspotify::Client::new(creds),
            is_initialized: false,
        })
    }

    pub async fn finish_initialization_from_cache(&mut self) -> error::Result<()> {
        let refresh_token = match std::fs::read_to_string(".refresh_token") {
            Ok(t) => t,
            Err(e) => {
                return Err(SoundbaseError {
                    msg: e.to_string(),
                    http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                });
            }
        };

        self.client.set_refresh_token(Some(refresh_token)).await;
        self.is_initialized = true;
        Ok(())
    }

    pub async fn get_authorization_url(&self) -> String {
        self.auth_url.clone()
    }

    pub async fn finish_initialization_with_code(&mut self, uri: &str) -> error::Result<()> {
        let full_uri = self.redirect_url.clone() + "?" + uri;
        println!("\tURI => {}", full_uri);
        match self.client.redirected(full_uri.as_str(), self.state.as_str()).await {
            Ok(_) => {
                match self.client.refresh_token().await {
                    Some(token) => {
                        if let Err(e) = std::fs::write(".refresh_token", token) {
                            Err(SoundbaseError {
                                msg: e.to_string(),
                                http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                            })
                        } else {
                            self.is_initialized = true;
                            Ok(())
                        }
                    }
                    None => {
                        Err(SoundbaseError::new("Couldn't get spotify refresh token for storage!"))
                    }
                }
            }
            Err(e) => {
                Err(SoundbaseError {
                    msg: e.to_string(),
                    http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                })
            }
        }
    }

    pub async fn publish_song_like(&self, song: &SpotifySong) -> error::Result<()>
    {
        if !self.is_initialized {
            return Err(SoundbaseError::new("Spotify connection not initialized. Call /spotify/start_auth to get authorization URL!"));
        }

        let tracks = [song.track_id.as_str()];
        match self.client.library().save_tracks(tracks.iter().clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("\tFailed to mark track as liked => {:?}", e);
                Err(SoundbaseError::new("Failed to mark track as liked"))
            }
        }
    }

    pub async fn publish_song_dislike(&self, song: &SpotifySong) -> error::Result<()>
    {
        if !self.is_initialized {
            return Err(SoundbaseError::new("Spotify connection not initialized. Call /spotify/start_auth to get authorization URL!"));
        }

        let tracks = [song.track_id.as_str()];
        match self.client.library().unsave_tracks(tracks.iter().clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("\tFailed to mark track as not liked => {:?}", e);
                Err(SoundbaseError::new("Failed to mark track as not liked"))
            }
        }
    }

    pub async fn find_song_in_spotify<DB>(&self, db: DB, song: &Song) -> Option<String>
        where DB: FollowForeignReference<Song, Artist> + FollowForeignReference<Song, Album>
    {
        if !self.is_initialized {
            println!("\tSpotify connection not initialized. Call /spotify/start_auth to get authorization URL!");
            return None;
        }

        let song_title = song.name.as_str();
        let artist: Artist = match db.follow_reference(song) {
            Ok(artist) => artist,
            Err(e) => {
                println!("Couldn't read Artist from DB! => {:?}", e);
                return None;
            }
        };
        let song_artist = artist.name.clone().replace("feat.", "");

        let song_album = if song.has_album_info() {
            let album: Album = match db.follow_reference(song) {
                Ok(album) => album,
                Err(e) => {
                    println!("\tCoulnt't read Album from DB! => {:?}", e);
                    return None;
                }
            };
            Some(album.name.clone())
        } else {
            None
        };

        self.find_song(song_title, song_artist.as_str(), song_album).await
    }

    async fn find_song(&self, song_title: &str, song_artist: &str, album: Option<String>) -> Option<String> {
        let mut query = song_title.to_owned() + " " + song_artist;
        let mut song_album = "".to_owned();
        match album {
            Some(ref a) => {
                query = query + " " + a.as_str();
                song_album = a.clone();
            }
            _ => {}
        }

        let types = [aspotify::ItemType::Track];

        println!("\tSearching for tracks with query => {}", query);

        match self.client.search().search(
            query.as_str(),
            types.iter().copied(),
            false,
            10,
            0,
            Some(aspotify::Market::FromToken)).await
        {
            Ok(results) => {
                match results.data.tracks {
                    Some(tracks) => {
                        let mut best_match = "".to_owned();
                        let mut best_score = 0.0;
                        for track in tracks.items {
                            let track_title = track.name.as_str();
                            let track_album = track.album.name.as_str();
                            let track_artist = track.artists.iter().fold("".to_owned(), |list, a| list + " " + a.name.as_str()).trim().to_owned();

                            let track_name_sim = strsim::jaro_winkler(song_title.to_uppercase().as_str(), track_title.to_uppercase().as_str());
                            let track_album_sim =
                                if album.is_some() { strsim::normalized_levenshtein(song_album.to_uppercase().as_str(), track_album.to_uppercase().as_str()) } else { 1.0 };
                            let track_artist_sim = strsim::normalized_damerau_levenshtein(song_artist.to_uppercase().as_str(), track_artist.to_uppercase().as_str());

                            let avg = (track_name_sim + track_album_sim + track_artist_sim) / 3.0;

                            println!("\tCalculated an avg score of {} for track [{}] {} - {} ({})", avg, track.id.clone().unwrap(), track_title, track_artist, track_album);
                            println!("\t\tTitle {}; Artist {}; Album {}", track_name_sim, track_artist_sim, track_album_sim);

                            if avg > best_score {
                                best_match = track.id.clone().unwrap();
                                best_score = avg;
                                println!("\tFound new best match with score {} for track [{}] {} - {} ({})",
                                         best_score, track.id.clone().unwrap(), track_title, track_artist, track_album);
                            }
                        }

                        if best_score > 0.75 {
                            println!("\tFound best match with score {} for track {}", best_score, best_match);
                            Some(best_match)
                        } else {
                            println!("\tDidn't find a match close enough to the input; the best result scored {} for {}", best_score, best_match);
                            None
                        }
                    }
                    None => {
                        println!("\tDidn't get tracks back for a query on tracks!");
                        return None;
                    }
                }
            }
            Err(e) => {
                println!("\tError while searching with the following query => {}; {:?}", query, e);
                return None;
            }
        }
    }

    pub async fn find_song_in_library(&self, song: &RawSong) -> Option<SpotifySong>
    {
        if !self.is_initialized {
            println!("\tSpotify connection not initialized. Call /spotify/start_auth to get authorization URL!");
            return None;
        }

        println!("\tTrying to find song {:?} in spotify library.", song);

        let album = if song.album.is_empty() { None } else { Some(song.album.clone()) };
        match self.find_song(song.title.as_str(), song.artist.as_str(), album).await {
            Some(trackid) => {
                //check if its in the library
                let tracks = [trackid.clone()];
                match self.client.library().user_saved_tracks(tracks.iter().clone()).await {
                    Ok(response) => {
                        let contains = &response.data;
                        assert_eq!(contains.len(), 1);
                        Some(SpotifySong {
                            track_id: trackid.clone(),
                            in_library: contains[0],
                        })
                    }
                    Err(e) => {
                        println!("\tFailed to read current state of track {} in library! ({:?})", trackid, e);
                        None
                    }
                }
            }
            None => {
                None
            }
        }
    }
}

