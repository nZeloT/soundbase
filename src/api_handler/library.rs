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

use serde::Serialize;
use warp::Reply;
use crate::db_new::album::AlbumDb;
use crate::db_new::artist::ArtistDb;
use crate::db_new::DbApi;
use crate::db_new::models::{Album, Artist, Track};
use crate::db_new::track::TrackDb;
use crate::error::Error;
use crate::model::{RequestPage, ResponsePage};
use crate::WebResult;

#[derive(Serialize, Debug)]
struct TrackListResponse {
    entries: Vec<Track>,

    #[serde(flatten)]
    page: ResponsePage,
}

impl TrackListResponse {
    pub fn new(data: Vec<Track>, page: &RequestPage) -> Self {
        let page = ResponsePage::new("/api/v1/library/track/", &page, data.len() == page.limit() as usize);
        Self {
            entries: data,
            page,
        }
    }
}

#[derive(Serialize, Debug)]
struct AlbumsListResponse {
    entries : Vec<Album>,

    #[serde(flatten)]
    page : ResponsePage
}

impl AlbumsListResponse {
    pub fn new(data : Vec<Album>, page : &RequestPage) -> Self {
        let page = ResponsePage::new("/api/v1/library/album/", &page, data.len() == page.limit() as usize);
        Self {
            entries: data,
            page,
        }
    }
}

#[derive(Serialize, Debug)]
struct ArtistListResponse {
    entries: Vec<Artist>,

    #[serde(flatten)]
    page : ResponsePage
}

impl ArtistListResponse {
    pub fn new(data : Vec<Artist>, page : &RequestPage) -> Self {
        let page = ResponsePage::new("/api/v1/library/artist/", &page, data.len() == page.limit() as usize);
        Self{
            entries: data,
            page,
        }
    }
}

pub async fn load_tracks(db: DbApi, page: RequestPage) -> WebResult<impl Reply> {
    let api: &dyn TrackDb = &db;
    let results = api.load_tracks(&page);
    match results {
        Ok(data) => Ok(warp::reply::json(&TrackListResponse::new(data, &page))),
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
        Ok(data) => Ok(warp::reply::json(&AlbumsListResponse::new(data, &page))),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_single_album(album_id : i32, db : DbApi) -> WebResult<impl Reply> {
    let api : &dyn AlbumDb = &db;
    match api.find_by_id(album_id) {
        Ok(album) => Ok(warp::reply::json(&album)),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_artists(db : DbApi, page : RequestPage) -> WebResult<impl Reply> {
    let api : &dyn ArtistDb = &db;
    match api.load_artists(&page) {
        Ok(data) => Ok(warp::reply::json(&ArtistListResponse::new(data, &page))),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}

pub async fn load_single_artist(artist_id : i32, db : DbApi) -> WebResult<impl Reply> {
    let api : &dyn ArtistDb = &db;
    match api.find_by_id(artist_id) {
        Ok(artist) => Ok(warp::reply::json(&artist)),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}