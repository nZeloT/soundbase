use rspotify::senum::SearchType;
use rspotify::senum::Country;
use rspotify::model::search::SearchResult;
use rspotify::model::page::Page;
use rspotify::model::track::FullTrack;
use crate::error;
use crate::error::SoundbaseError;
use crate::db::song::Song;
use crate::db::FollowForeignReference;
use crate::db::artist::Artist;
use crate::db::album::Album;
use crate::db::db_error::DbError;
use crate::model::song_like::RawSong;

const NECESSARY_SCOPES: &str = "user-library-read user-library-modify user-read-private";

pub struct SpotifySong {
    track_id: String,
    in_library: bool,
}

#[derive(Debug)]
pub struct Spotify {
    oauth: rspotify::oauth2::SpotifyOAuth,
    client: rspotify::client::Spotify,
    is_initialized: bool,
}

impl Spotify {
    pub fn new() -> Self {
        Spotify {
            oauth: rspotify::oauth2::SpotifyOAuth::default().scope(NECESSARY_SCOPES).build(),
            client: rspotify::client::Spotify::default(),
            is_initialized: false,
        }
    }

    pub async fn finish_initialization_from_cache(&mut self) -> error::Result<()> {
        match self.oauth.get_cached_token().await {
            Some(token) => {
                let client_creds = rspotify::oauth2::SpotifyClientCredentials::default().token_info(token).build();
                self.client = rspotify::client::Spotify::default().client_credentials_manager(client_creds).build();
                self.is_initialized = true;
                Ok(())
            }
            None => Err(SoundbaseError::new("Failed to create token from cache! Needs manual authorization first."))
        }
    }

    pub async fn request_authorization_token(&self) -> String {
        let state = rspotify::util::generate_random_string(16);
        self.oauth.get_authorize_url(Some(&state), None)
    }

    pub async fn finish_initialization_with_code(&mut self, code: &str) -> error::Result<()> {
        match self.oauth.get_access_token(code).await {
            Some(token) => {
                let client_creds = rspotify::oauth2::SpotifyClientCredentials::default().token_info(token).build();
                self.client = rspotify::client::Spotify::default().client_credentials_manager(client_creds).build();
                self.is_initialized = true;
                Ok(())
            }
            None => Err(SoundbaseError::new("Failed to create token from authorization code!"))
        }
    }

    pub async fn publish_song_like<DB>(&self, mut db: DB, song: &Song)
        where DB: FollowForeignReference<Song, Artist> + FollowForeignReference<Song, Album>
    {
        if !self.is_initialized {
            println!("Spotify connection not initialized. Call /spotify/start_auth to get authorization URL!");
            return
        }

        match self.find_song_in_spotify(db, song).await {
            Some(track_id) => {
                let tracks = [track_id];
                match self.client.current_user_saved_tracks_add(&tracks).await {
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

    pub async fn publish_song_dislike<DB>(&self, mut db: DB, song: &Song)
        where DB: FollowForeignReference<Song, Artist> + FollowForeignReference<Song, Album>
    {
        if !self.is_initialized {
            println!("Spotify connection not initialized. Call /spotify/start_auth to get authorization URL!");
            return
        }

        match self.find_song_in_spotify(db, song).await {
            Some(track_id) => {
                let tracks = [track_id];
                match self.client.current_user_saved_tracks_delete(&tracks).await {
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
            return None
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

        match self.client.search(query.as_str(), SearchType::Track, 10, 0, Some(Country::Germany), None).await {
            Ok(results) => {
                match results {
                    SearchResult::Tracks(tracks) => {
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
                                best_match = track.uri.clone();
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
                    _ => {
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
            return None
        }

        let album = if song.album.is_empty() { None } else { Some(song.album.clone()) };
        match self.find_song(song.title.as_str(), song.artist.as_str(), album).await {
            Some(trackid) => {
                //check if its in the library
                let tracks = [trackid.clone()];
                match self.client.current_user_saved_tracks_contains(&tracks).await {
                    Ok(contains) => {
                        assert_eq!(contains.len(), 1);
                        Some(SpotifySong {
                            track_id: trackid.clone(),
                            in_library: contains[0],
                        })
                    },
                    Err(e) => {
                        println!("Failed to read current state of track {} in library! ({:?})", trackid, e);
                        None
                    }
                }
            },
            None => {
                None
            }
        }
    }
}

