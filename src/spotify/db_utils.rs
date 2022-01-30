use rspotify::model::{FullAlbum, FullArtist, FullTrack};
use crate::db_new::album::AlbumDb;
use crate::db_new::artist::ArtistDb;
use crate::db_new::models::{Album, Artist, NewAlbum, NewArtist, NewTrack, Track};
use crate::db_new::track::TrackDb;
use crate::model::{AlbumType, UniversalId};
use crate::Result;

pub fn get_or_create_album(api : &impl AlbumDb, spotify_album : &FullAlbum) -> Result<Album> {
    let id = UniversalId::Spotify(spotify_album.id.to_string());
    let album = match api.find_by_universal_id(&id)? {
        Some(album) => album,
        None => api.new_full_album(NewAlbum {
            name: &*spotify_album.name,
            year: spotify_album.release_date[..4].parse::<i32>()?,
            total_tracks: spotify_album.tracks.total as i32,
            is_known_spot: true,
            is_known_local: false,
            was_aow: Some(false),
            is_faved: Some(false),
            spot_id: Some(spotify_album.id.to_string()),
            album_type: Some(AlbumType::from(spotify_album.album_type).into()),
        })?
    };
    Ok(album)
}

pub fn get_or_create_artist(api : &impl ArtistDb, spotify_artist : &FullArtist) -> Result<Artist> {
    let id = UniversalId::Spotify(spotify_artist.id.to_string());
    let db_artist = match api.find_artist_by_universal_id(&id)? {
        Some(artist) => artist,
        None => api.new_full_artist(NewArtist {
            name: &*spotify_artist.name,
            spot_id: Some(spotify_artist.id.to_string()),
            is_known_local: false,
            is_known_spot: true
        })?
    };
    Ok(db_artist)
}

pub fn get_or_create_track(api : &impl TrackDb, db_album : &Album, spotify_track : &FullTrack) -> Result<Track> {
    let id = UniversalId::Spotify(spotify_track.id.clone().unwrap().to_string());
    let db_track = match api.find_track_by_universal_id(&id)? {
        Some(track) => track,
        None => api.new_full_track(NewTrack {
            title: &*spotify_track.name,
            is_faved: false,
            album_id: db_album.album_id,
            track_number: Some(spotify_track.track_number as i32),
            disc_number: Some(spotify_track.disc_number as i32),
            local_file: None,
            duration_ms: spotify_track.duration.as_millis() as i64,
            spot_id: Some(spotify_track.id.as_ref().unwrap().to_string()),
        })?
    };
    Ok(db_track)
}