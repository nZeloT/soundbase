use aspotify::Scope;

use crate::error;
use crate::error::SoundbaseError;
use crate::db::song::Song;
use crate::db::FollowForeignReference;
use crate::db::artist::Artist;
use crate::db::album::Album;
use crate::model::song_like::RawSong;

const NECESSARY_SCOPES: [Scope; 3] = [Scope::UserLibraryRead, Scope::UserFollowModify, Scope::UserReadPrivate];

pub struct SpotifySong {
    track_id: String,
    in_library: bool,
}

#[derive(Debug)]
pub struct Spotify {
    state: String,
    client: aspotify::Client,
    is_initialized: bool,
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
        Ok(Spotify {
            state: String::new(),
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

    pub async fn request_authorization_token(&mut self) -> String {
        let redirect_uri = match std::env::var("REDIRECT_URI") {
            Ok(uri) => uri,
            Err(_) => "http://some.uri".to_string()
        };
        let (url, state) = aspotify::authorization_url(
            &self.client.credentials.id,
            NECESSARY_SCOPES.iter().copied(),
            false,
            redirect_uri.as_str(),
        );
        self.state = state;
        url
    }

    pub async fn finish_initialization_with_code(&mut self, uri: &str) -> error::Result<()> {
        println!("URI => {}", uri);
        match self.client.redirected(uri, self.state.as_str()).await {
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

    pub async fn publish_song_like<DB>(&self, db: DB, song: &Song)
        where DB: FollowForeignReference<Song, Artist> + FollowForeignReference<Song, Album>
    {
        if !self.is_initialized {
            println!("Spotify connection not initialized. Call /spotify/start_auth to get authorization URL!");
            return;
        }

        match self.find_song_in_spotify(db, song).await {
            Some(track_id) => {
                let tracks = [track_id];
                match self.client.library().save_tracks(tracks.iter().clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        println!("Failed to mark track as liked => {:?}", e)
                    }
                }
            }
            None => {
                println!("Couldn't find spotify track for given song => {:?}", song)
            }
        }
    }

    pub async fn publish_song_dislike<DB>(&self, db: DB, song: &Song)
        where DB: FollowForeignReference<Song, Artist> + FollowForeignReference<Song, Album>
    {
        if !self.is_initialized {
            println!("Spotify connection not initialized. Call /spotify/start_auth to get authorization URL!");
            return;
        }

        match self.find_song_in_spotify(db, song).await {
            Some(track_id) => {
                let tracks = [track_id];
                match self.client.library().unsave_tracks(tracks.iter().clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        println!("Failed to mark track as not liked => {:?}", e)
                    }
                }
            }
            None => {
                println!("Couldn't find spotify track for given song => {:?}", song)
            }
        }
    }

    pub async fn find_song_in_spotify<DB>(&self, mut db: DB, song: &Song) -> Option<String>
        where DB: FollowForeignReference<Song, Artist> + FollowForeignReference<Song, Album>
    {
        if !self.is_initialized {
            println!("Spotify connection not initialized. Call /spotify/start_auth to get authorization URL!");
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
                    println!("Coulnt't read Album from DB! => {:?}", e);
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
                            let track_artist = track.artists.iter().fold("".to_owned(), |list, a| list + " " + a.name.as_str());

                            let track_name_sim = 1.0 - strsim::jaro_winkler(song_title, track_title);
                            let track_album_sim = if album.is_some() { strsim::normalized_levenshtein(song_album.as_str(), track_album) } else { 1.0 };
                            let track_artist_sim = strsim::sorensen_dice(song_artist, track_artist.as_str());

                            let avg = (track_name_sim + track_album_sim + track_artist_sim) / 3.0;

                            println!("Calculated an avg score of {} for track {:?}", avg, track);

                            if avg > best_score {
                                best_match = track.id.clone().unwrap();
                                best_score = avg;
                                println!("Found new best match with score {} for track {:?}", best_score, track);
                            }
                        }

                        if best_score > 0.75 {
                            println!("Found best match with score {} for track {}", best_score, best_match);
                            Some(best_match)
                        } else {
                            println!("Didn't find a match close enough to the input; the best result scored {} for {}", best_score, best_match);
                            None
                        }
                    }
                    None => {
                        println!("Didn't get tracks back for a query on tracks!");
                        return None;
                    }
                }
            }
            Err(e) => {
                println!("Error while searching with the following query => {}; {:?}", query, e);
                return None;
            }
        }
    }

    pub async fn find_song_in_library(&self, song: &RawSong) -> Option<SpotifySong>
    {
        if !self.is_initialized {
            println!("Spotify connection not initialized. Call /spotify/start_auth to get authorization URL!");
            return None;
        }

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
                        println!("Failed to read current state of track {} in library! ({:?})", trackid, e);
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

