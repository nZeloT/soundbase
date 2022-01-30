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

/***
/api/v1/library/track?offset=&limit=
                      /<id>
/api/v1/library/album/?offset=&limit=
                      /<id>
                      /tracks
/api/v1/library/artist/?offset=&limit=
                       /<id>
 */

use warp::Reply;

use crate::db_new::album::AlbumDb;
use crate::db_new::album_artist::AlbumArtistsDb;
use crate::db_new::artist::ArtistDb;
use crate::db_new::artist_genre::ArtistGenreDb;
use crate::db_new::DbApi;
use crate::db_new::track::TrackDb;
use crate::error::Error;
use crate::model::RequestPage;
use crate::WebResult;

pub async fn load_tracks(db: DbApi, page: RequestPage) -> WebResult<impl Reply> {
    let api: &dyn TrackDb = &db;
    let results = api.load_tracks(&page);
    match results {
        Ok(data) => Ok(warp::reply::json(&library_models::TrackListResponse::new(data, &page))),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_single_track(track_id: i32, db: DbApi) -> WebResult<impl Reply> {
    let api: &dyn TrackDb = &db;
    let result = api.find_by_id(track_id);
    match result {
        Ok(track) => Ok(warp::reply::json(&track)),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_albums(db : DbApi, page : RequestPage) -> WebResult<impl Reply> {
    let api : &dyn AlbumDb = &db;
    let result = api.load_albums(&page);
    match result {
        Ok(data) => Ok(warp::reply::json(&library_models::AlbumsListResponse::new(data, &page))),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_single_album(album_id : i32, db : DbApi) -> WebResult<impl Reply> {
    let api : &dyn AlbumDb = &db;
    match api.find_by_id(album_id) {
        Ok(album) => {
            let api : &dyn TrackDb = &db;
            let tracks = match api.load_tracks_for_album(&album) {
                Ok(tracks) => tracks,
                Err(e) => return Err(warp::reject::custom(Error::DatabaseError(e)))
            };
            let api : &dyn AlbumArtistsDb = &db;
            let artists = match api.load_artists_for_album(&album) {
                Ok(artists) => artists,
                Err(e)  => return Err(warp::reject::custom(Error::DatabaseError(e)))
            };
            let full_album = library_models::FullAlbum::new(album, tracks, artists);

            Ok(warp::reply::json(&full_album))
        },
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_artists(db : DbApi, page : RequestPage) -> WebResult<impl Reply> {
    let api : &dyn ArtistDb = &db;
    match api.load_artists(&page) {
        Ok(data) => Ok(warp::reply::json(&library_models::ArtistListResponse::new(data, &page))),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_single_artist(artist_id : i32, db : DbApi) -> WebResult<impl Reply> {
    let api : &dyn ArtistDb = &db;
    match api.find_by_id(artist_id) {
        Ok(artist) => {
            let api : &dyn TrackDb = &db;
            let tracks = match api.load_fav_tracks_for_artist(&artist, &RequestPage::new(0, 10)) {
                Ok(t) => t,
                Err(e) => return Err(warp::reject::custom(Error::DatabaseError(e)))
            };

            let api : &dyn ArtistGenreDb = &db;
            let genres = match api.load_genres_for_artist(&artist) {
                Ok(g) => g,
                Err(e) => return Err(warp::reject::custom(Error::DatabaseError(e)))
            };

            let api : &dyn AlbumArtistsDb = &db;
            let albums = match api.load_albums_for_artist(&artist) {
                Ok(a) => a,
                Err(e) => return Err(warp::reject::custom(Error::DatabaseError(e)))
            };
            let full_artist = library_models::FullArtist::new(artist, genres, albums, tracks);
            Ok(warp::reply::json(&full_artist))
        },
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

fn library_prefix(str : &str) -> String {
    let s = "library/".to_string() + str;
    super::path_prefix(&s)
}

mod library_models {
    use serde::Serialize;

    use crate::db_new::models;
    use crate::model::{RequestPage, ResponsePage};

    #[derive(Serialize, Debug)]
    pub struct TrackListResponse {
        entries: Vec<models::Track>,

        #[serde(flatten)]
        page: ResponsePage,
    }

    impl TrackListResponse {
        pub fn new(data: Vec<models::Track>, page: &RequestPage) -> Self {
            let page = ResponsePage::new(&super::library_prefix("track/"), &page, data.len() == page.limit() as usize);
            Self {
                entries: data,
                page,
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub struct FlatAlbum {
        id : i32,
        name : String,
        year : i32,
        is_faved : bool,
        is_known_spot : bool,
        is_known_local : bool
    }

    #[derive(Serialize, Debug)]
    pub struct AlbumsListResponse {
        entries : Vec<FlatAlbum>,

        #[serde(flatten)]
        page : ResponsePage
    }

    impl AlbumsListResponse {
        pub fn new(data : impl IntoIterator<Item=impl Into<FlatAlbum>>, page : &RequestPage) -> Self {
            let entries : Vec<FlatAlbum> = data.into_iter().map(|e|e.into()).collect();
            let page = ResponsePage::new(&super::library_prefix("album/"), &page, entries.len() == page.limit() as usize);
            Self {
                entries,
                page,
            }
        }
    }

    impl From<models::Album> for FlatAlbum {
        fn from(source : models::Album) -> Self {
            Self {
                id: source.album_id,
                name: source.name,
                year: source.year,
                is_faved: source.is_faved,
                is_known_local: source.is_known_local,
                is_known_spot: source.is_known_spot
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub struct FullAlbum {
        #[serde(flatten)]
        flat : FlatAlbum,

        was_aow : bool,
        tracks : Vec<models::Track>,
        artists : Vec<FlatArtist>
    }

    impl FullAlbum {
        pub fn new(album : models::Album,
                   tracks : impl IntoIterator<Item=impl Into<models::Track>>,
                   artists : impl IntoIterator<Item=impl Into<FlatArtist>>) -> Self {
            Self {
                flat: album.clone().into(),
                was_aow: album.was_aow,
                tracks: tracks.into_iter().map(|e|e.into()).collect(),
                artists: artists.into_iter().map(|e|e.into()).collect(),
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub struct FlatArtist {
        id : i32,
        name : String,
        is_known_local : bool,
        is_known_spot : bool,
        is_faved : bool,
    }

    #[derive(Serialize, Debug)]
    pub struct ArtistListResponse {
        entries: Vec<FlatArtist>,

        #[serde(flatten)]
        page : ResponsePage
    }

    impl ArtistListResponse {
        pub fn new(data : impl IntoIterator<Item=impl Into<FlatArtist>>, page : &RequestPage) -> Self {
            let entries : Vec<FlatArtist> = data.into_iter().map(|e|e.into()).collect();
            let page = ResponsePage::new(&super::library_prefix("artist/"), &page, entries.len() == page.limit() as usize);
            Self{
                entries,
                page,
            }
        }
    }

    impl From<models::Artist> for FlatArtist {
        fn from(source: models::Artist) -> Self {
            Self {
                id: source.artist_id,
                name: source.name,
                is_faved: source.is_faved,
                is_known_spot: source.is_known_spot,
                is_known_local: source.is_known_local
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub struct FullArtist {
        #[serde(flatten)]
        flat : FlatArtist,

        albums : Vec<FlatAlbum>,
        genres : Vec<models::Genre>,
        fav_tracks : Vec<models::Track>
    }

    impl FullArtist {
        pub fn new(flat : impl Into<FlatArtist>,
                   genres : impl IntoIterator<Item=impl Into<models::Genre>>,
                   albums : impl IntoIterator<Item=impl Into<FlatAlbum>>,
                   fav_tracks : impl IntoIterator<Item=models::Track>) -> Self {
            Self {
                flat: flat.into(),
                albums: albums.into_iter().map(|e|e.into()).collect(),
                genres: genres.into_iter().map(|e| e.into()).collect(),
                fav_tracks: fav_tracks.into_iter().map(|e|e.into()).collect()
            }
        }
    }
}