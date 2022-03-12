use itertools::Itertools;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use super::definition::library_server::{Library};
use super::definition::{
    LibraryEntityRequest,
    ListEntitiesRequest,
    FavStateRequest,
    LibraryEntityResponse,
    SimpleLibraryEntityResponse,
    Blank,
};
use crate::db_new;
use crate::db_new::album::AlbumDb;
use crate::db_new::artist::ArtistDb;
use crate::db_new::{DbApi, SetFavedState};
use crate::db_new::album_artist::AlbumArtistsDb;
use crate::db_new::models::{Album, Artist, Track};
use crate::db_new::track::TrackDb;
use crate::db_new::track_artist::TrackArtistsDb;
use crate::model::RequestPage;

pub struct LibraryService {
    pub(crate) db: DbApi,
}

#[tonic::async_trait]
impl Library for LibraryService {
    async fn get(&self, request: Request<LibraryEntityRequest>) -> Result<Response<LibraryEntityResponse>, Status> {
        use super::definition::library_entity_response::LibraryEntities;
        use super::definition::{FullArtist, FullTrack};
        let id: i32 = request.get_ref().id;
        match request.get_ref().entity() {
            super::definition::LibraryEntities::Artist => {
                let api: &dyn ArtistDb = &self.db;
                match api.find_by_id(id)? {
                    Some(artist) => {
                        let api : &dyn AlbumArtistsDb = &self.db;
                        let albums = api.load_albums_for_artist(&artist)?;
                        Ok(Response::new(LibraryEntityResponse {
                            library_entities: Some(LibraryEntities::Artist(FullArtist::from_db(&artist, &albums)))
                        }))
                    },
                    None => Err(Status::not_found("Artist not found!"))
                }
            }
            super::definition::LibraryEntities::Album => {
                match load_full_album(&self.db, id)? {
                    Some(album) => {
                        Ok(Response::new(LibraryEntityResponse {
                            library_entities: Some(LibraryEntities::Album(album))
                        }))
                    },
                    None => Err(Status::not_found("Album not found!"))
                }
            }
            super::definition::LibraryEntities::Track => {
                let api: &dyn TrackDb = &self.db;
                match api.find_by_id(id)? {
                    Some(track) => {
                        let api : &dyn AlbumDb = &self.db;
                        let album = api.load_album_for_track(&track)?;
                        let api : &dyn TrackArtistsDb = &self.db;
                        let artists = api.load_artists_for_track(&track)?;

                        Ok(Response::new(LibraryEntityResponse {
                            library_entities: Some(LibraryEntities::Track(FullTrack::from_db(&track, &album, &artists)))
                        }))
                    },
                    None => Err(Status::not_found("Track not found!"))
                }
            }
            _ => panic!("Entity not supported!")
        }
    }

    type ListStream = ReceiverStream<Result<SimpleLibraryEntityResponse, Status>>;

    async fn list(&self, request: Request<ListEntitiesRequest>) -> Result<Response<Self::ListStream>, Status> {
        use super::definition::simple_library_entity_response::LibraryEntities;
        let offset = request.get_ref().offset;
        let limit = request.get_ref().limit;
        let entities: Vec<SimpleLibraryEntityResponse> = match request.get_ref().entity() {
            super::definition::LibraryEntities::Artist => {
                let api: &dyn ArtistDb = &self.db;
                match api.load_artists(&RequestPage::new(offset as i64, limit as i64)) {
                    Ok(artists) => {
                        artists.iter()
                            .map(|artist| SimpleLibraryEntityResponse {
                                library_entities: Some(LibraryEntities::Artist(artist.into()))
                            })
                            .collect_vec()
                    }
                    Err(e) => return Err(Status::internal(e.to_string()))
                }
            }
            super::definition::LibraryEntities::Album => {
                let api: &dyn AlbumDb = &self.db;
                match api.load_albums(&RequestPage::new(offset as i64, limit as i64)) {
                    Ok(albums) => {
                        albums.iter()
                            .map(|albums| SimpleLibraryEntityResponse {
                                library_entities: Some(LibraryEntities::Album(albums.into()))
                            })
                            .collect_vec()
                    }
                    Err(e) => return Err(Status::internal(e.to_string()))
                }
            }
            super::definition::LibraryEntities::Track => {
                load_track_list(&self.db, &RequestPage::new(offset as i64, limit as i64))?
                    .iter()
                    .map(|track| SimpleLibraryEntityResponse {
                        library_entities: Some(LibraryEntities::Track(track.clone()))
                    }).collect_vec()
            }
            _ => panic!("Entity not supported!")
        };

        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            for entity in &entities {
                tx.send(Ok(entity.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn set_fav_state(&self, request: Request<FavStateRequest>) -> Result<Response<Blank>, Status> {
        use super::definition::LibraryEntities;
        let id = request.get_ref().to_owned().entity.unwrap().id;
        let target_state = request.get_ref().new_fav_state;
        let result = match request.get_ref().to_owned().entity.unwrap().entity() {
            LibraryEntities::Artist => {
                let api: &dyn SetFavedState<Artist> = &self.db;
                api.set_faved_state(id, target_state)
            }
            LibraryEntities::Album => {
                let api: &dyn SetFavedState<Album> = &self.db;
                api.set_faved_state(id, target_state)
            }
            LibraryEntities::Track => {
                let api: &dyn SetFavedState<Track> = &self.db;
                api.set_faved_state(id, target_state)
            }
            _ => panic!("Entity not supported!")
        };

        match result {
            Ok(_) => Ok(Response::new(super::definition::Blank {})),
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }
}

fn load_full_album(db : &DbApi, album_id : i32) -> Result<Option<super::definition::FullAlbum>, db_new::DbError> {
    let api : &dyn AlbumDb = db;
    match api.find_by_id(album_id)? {
        Some(album) => {
            let album_artists = db.load_artists_for_album(&album)?;
            let tracks = db.load_tracks_for_album(&album)?;
            let track_artist_map = db.load_artist_ids_for_tracks(&tracks)?;

            //1. collect missing artist ids
            let missing_artists = track_artist_map.values().flatten().unique().filter(|artist_id|{
                //try to find it in loaded artists
                !album_artists.iter().any(|artist| artist.artist_id == **artist_id)
            }).cloned().collect_vec();

            //2. load missing artists if any
            let api : &dyn ArtistDb = db;
            let mut missing_artists = if !missing_artists.is_empty() {
                api.find_by_ids(missing_artists)?
            }else{
                vec![]
            };

            //3. map album artists
            let simple_album_artists = album_artists.iter().map(super::definition::SimpleArtist::from).collect_vec();

            //4. map tracks
            let mut track_artists = album_artists;
            track_artists.append(&mut missing_artists);

            let track_album = super::definition::SimpleAlbum::from(&album);
            let simple_album_tracks = tracks.iter()
                .sorted_by(|a, b| Ord::cmp(&a.track_number, &b.track_number))
                .map(|track| {
                    let album = track_album.clone();
                    let artists = track_artist_map.get(&track.track_id).unwrap()
                        .iter()
                        .map(|artist_id| track_artists.iter().find(|artist| artist.artist_id == *artist_id).unwrap())
                        .map(super::definition::SimpleArtist::from)
                        .collect_vec();

                    super::definition::SimpleTrack::from_db(track, album, artists)
                }).collect_vec();

            Ok(Some(super::definition::FullAlbum::from_db(&album, simple_album_artists, simple_album_tracks)))
        },
        None => Ok(None)
    }
}

fn load_track_list(db : &DbApi, page : &RequestPage) -> Result<Vec<super::definition::SimpleTrack>, db_new::DbError> {
    let tracks = db.load_tracks(page)?;

    let albums = db.load_albums_for_tracks(&tracks)?;

    let track_artist_ids = db.load_artist_ids_for_tracks(&tracks)?;

    let api : &dyn ArtistDb = db;
    let ids_to_load = track_artist_ids.values().flatten().unique().cloned().collect_vec();
    let artists = api.find_by_ids(ids_to_load)?;

    let simple_tracks = tracks.iter().map(|track| {
        let album = albums.iter().find(|a| a.album_id == track.album_id).unwrap();
        let artists = track_artist_ids.get(&track.track_id).unwrap()
            .iter()
            .map(|artist_id| artists.iter().find(|artist| artist.artist_id == *artist_id).unwrap())
            .map(super::definition::SimpleArtist::from)
            .collect_vec();

        super::definition::SimpleTrack::from_db(&track, super::definition::SimpleAlbum::from(album), artists)
    }).collect_vec();

    Ok(simple_tracks)
}

impl super::definition::FullArtist {
    fn from_db(db_artist: &Artist, db_albums : &[Album]) -> Self {
        let simple_albums = db_albums
            .iter()
            .map(super::definition::SimpleAlbum::from)
            .collect_vec();
        Self {
            artist_id: db_artist.artist_id,
            name: db_artist.name.clone(),
            is_faved: db_artist.is_faved,
            albums: simple_albums
        }
    }
}

impl super::definition::FullAlbum {
    fn from_db(db_album: &Album, album_artists : Vec<super::definition::SimpleArtist>,
               album_tracks : Vec<super::definition::SimpleTrack>) -> Self {
        Self {
            album_id: db_album.album_id,
            name: db_album.name.clone(),
            artists: album_artists,
            tracks: album_tracks,
            album_type: super::definition::AlbumTypes::Album.into(), //FIXME
            year: db_album.year,
            track_count: db_album.total_tracks,
            is_faved: db_album.is_faved,
            was_album_of_week: db_album.was_aow,
        }
    }
}

impl super::definition::FullTrack {
    fn from_db(db_track: &Track, db_album : &Album, db_artists : &[Artist]) -> Self {
        let simple_artists = db_artists
            .iter()
            .map(super::definition::SimpleArtist::from)
            .collect_vec();
        Self {
            track_id: db_track.track_id,
            title: db_track.title.clone(),
            album: Some(db_album.into()),
            artists: simple_artists,
            track_number: db_track.track_number,
            disc_number: db_track.disc_number,
            duration_ms: db_track.duration_ms,
            is_faved: db_track.is_faved,
        }
    }
}

impl From<&db_new::models::Artist> for super::definition::SimpleArtist {
    fn from(db_artist: &db_new::models::Artist) -> Self {
        Self {
            artist_id: db_artist.artist_id,
            name: db_artist.name.clone(),
        }
    }
}

impl From<&db_new::models::Album> for super::definition::SimpleAlbum {
    fn from(db_album: &db_new::models::Album) -> Self {
        Self {
            album_id: db_album.album_id,
            name: db_album.name.clone(),
        }
    }
}

impl super::definition::SimpleTrack {
    fn from_db(db_track: &db_new::models::Track, album : super::definition::SimpleAlbum, artists : Vec<super::definition::SimpleArtist>) -> Self {
        Self {
            track_id: db_track.track_id,
            title: db_track.title.clone(),
            album : Some(album),
            artists,
            is_faved : db_track.is_faved,
            duration_ms : db_track.duration_ms
        }
    }
}

impl From<db_new::DbError> for Status {
    fn from(error: db_new::DbError) -> Self {
        Status::internal(error.to_string())
    }
}