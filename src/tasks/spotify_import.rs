use std::collections::HashMap;

use itertools::Itertools;
use rspotify::model::{ArtistId, FullAlbum, FullArtist, FullTrack};

use crate::db_new::album::AlbumDb;
use crate::db_new::album_artist::AlbumArtistsDb;
use crate::db_new::artist::ArtistDb;
use crate::db_new::DbApi;
use crate::db_new::models::{Album, Artist, Track};
use crate::db_new::track::TrackDb;
use crate::db_new::track_artist::TrackArtistsDb;
use crate::model::RequestPage;
use crate::Result;
use crate::spotify::db_utils::{get_or_create_album, get_or_create_artist, get_or_create_track};
use crate::SpotifyApi;

pub struct SpotifyImporter {
    db: DbApi,
    spotify: SpotifyApi,
    known_artists: HashMap<String, Artist>,
    known_albums: HashMap<String, Album>,
    known_tracks: HashMap<String, Track>,
}

impl SpotifyImporter {
    pub fn new(db: DbApi, spotify: SpotifyApi) -> Self {
        Self {
            db,
            spotify,
            known_artists: HashMap::new(),
            known_albums: HashMap::new(),
            known_tracks: HashMap::new(),
        }
    }

    pub async fn do_import(&mut self) -> Result<()> {
        println!("Importing artists ...");
        self.import_artist_follows().await?;
        println!("Importing albums ...");
        self.import_faved_albums().await?;
        println!("Importing tracks ...");
        self.import_faved_tracks().await?;
        Ok(())
    }

    async fn import_artist_follows(&mut self) -> Result<()> {
        let mut handled = 0;
        let mut last: Option<String> = None;

        loop {
            let (total, follows) = self.spotify.get_saved_artists(last.as_deref(), Some(50)).await?;
            handled += follows.len();
            last = follows.last().map(|l| l.id.as_ref().to_string());

            for follow in &follows {
                self.import_artist(follow)?;
            }

            std::thread::sleep(std::time::Duration::from_secs(10));

            if handled == total as usize {
                break;
            }
        }
        Ok(())
    }

    fn import_artist(&mut self, artist: &FullArtist) -> Result<()> {
        if !self.known_artists.contains_key(&artist.id.as_ref().to_string()) {
            println!("Adding new artist {}", artist.name);
            let dba = get_or_create_artist(&self.db, artist)?;
            let api: &dyn ArtistDb = &self.db;
            api.set_faved_state(dba.artist_id, true)?;
            self.known_artists.insert(artist.id.to_string(), dba);
        }
        Ok(())
    }

    async fn import_faved_albums(&mut self) -> Result<()> {
        let mut current_offset = 0;
        loop {
            let (total, albums) = self.spotify.get_saved_albums(&RequestPage::new(current_offset, 50)).await?;
            current_offset += albums.len() as i64;

            for album in &albums {
                self.import_saved_album(album).await?;
            }

            std::thread::sleep(std::time::Duration::from_secs(15));

            if current_offset == total as i64 {
                break;
            }
        }
        Ok(())
    }

    async fn import_saved_album(&mut self, album: &FullAlbum) -> Result<()> {
        if !self.known_albums.contains_key(&album.id.as_ref().to_string()) {
            let dba = self.import_album(album).await?;
            let api: &dyn AlbumDb = &self.db;
            api.set_faved_state(dba.album_id, true)?;
        }
        Ok(())
    }

    async fn import_faved_tracks(&mut self) -> Result<()> {
        let mut current_offset = 0;
        loop {
            let (total, tracks) = self.spotify.get_saved_tracks(&RequestPage::new(current_offset, 50)).await?;
            current_offset += tracks.len() as i64;
            for track in tracks {
                self.import_track(track).await?;
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
            if current_offset == total as i64 {
                break;
            }
        }
        Ok(())
    }

    async fn import_track(&mut self, track : FullTrack) -> Result<()> {
        let track = match track.linked_from {
            Some(link) => {
                println!("Resolving linked track {}; given: {}, linked: {}", track.name, track.id.unwrap().to_string(), link.id.to_string());
                self.spotify.get_track(&link.id).await?
            },
            None => track
        };

        #[allow(clippy::map_entry)]
        if !self.known_tracks.contains_key(&track.id.as_ref().unwrap().to_string()) {
            println!("Adding new track {}", track.name);
            //check for corresponding album
            let album = match self.known_albums.get(&track.album.id.as_ref().unwrap().to_string()) {
                Some(album) => album,
                None => {
                    let full_album = self.spotify.get_album(track.album.id.as_ref().unwrap()).await?;
                    let _ = self.import_album(&full_album).await?;
                    self.known_albums.get(&track.album.id.as_ref().unwrap().to_string()).unwrap()
                }
            };

            let db_track = get_or_create_track(&self.db, album, &track)?;
            let api : &dyn TrackDb = &self.db;
            let _ = api.set_faved_state(db_track.track_id, true)?;

            let unknown_artists = track.artists.iter()
                .filter(|&a|!self.known_artists.contains_key(&a.id.as_ref().unwrap().to_string()))
                .unique_by(|&a|a.id.as_ref().unwrap().to_string())
                .map(|a|a.id.clone().unwrap())
                .collect::<Vec<ArtistId>>();

            if !unknown_artists.is_empty() {
                self.add_unknown_artists(&unknown_artists).await?;
            }

            let track_id = db_track.track_id;
            let api : &dyn TrackArtistsDb = &self.db;
            for artist in &track.artists {
                let artist_id = self.known_artists.get(&artist.id.as_ref().unwrap().to_string()).unwrap().artist_id;
                let _ = api.new_track_artist_if_missing(track_id, artist_id)?;
            }

            self.known_tracks.insert(track.id.as_ref().unwrap().to_string(), db_track);
        }
        Ok(())
    }

    async fn import_album(&mut self, album : &FullAlbum) -> Result<Album> {
        println!("Adding new album {}", album.name);
        let dba = get_or_create_album(&self.db, album)?;

        let unknown_artists = album.artists.iter().filter(|&a| {
            !self.known_artists.contains_key(&a.id.as_ref().unwrap().to_string())
        }).unique_by(|&a| a.id.as_ref().unwrap().to_string())
            .map(|a| a.id.as_ref().unwrap().clone())
            .collect::<Vec<ArtistId>>();

        if !unknown_artists.is_empty() {
            self.add_unknown_artists(&unknown_artists).await?;
        }

        //link albums to artists
        let album_id = dba.album_id;
        let api: &dyn AlbumArtistsDb = &self.db;
        for artist in &album.artists {
            let artist_id = self.known_artists.get(&artist.id.as_ref().unwrap().to_string()).unwrap().artist_id;
            let _ = api.new_album_artist_if_missing(artist_id, album_id)?;
        }
        self.known_albums.insert(album.id.to_string(), dba.clone());
        Ok(dba)
    }

    async fn add_unknown_artists(&mut self, artists : &[ArtistId]) -> Result<()> {
        let full_unknown_artists = self.spotify.get_artists(artists).await?;
        for unknown in &full_unknown_artists {
            println!("Adding new artist {}", unknown.name);
            let artist = get_or_create_artist(&self.db, unknown)?;
            self.known_artists.insert(unknown.id.to_string(), artist);
        }
        Ok(())
    }
}