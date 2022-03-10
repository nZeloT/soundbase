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

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum UniversalId {
    Spotify(String),
    Database(i32),
}

impl From<&str> for UniversalId {
    fn from(input: &str) -> Self {
        match &input[..1] {
            "s" => UniversalId::Spotify(input.to_string()),
            _ => UniversalId::Database(input.parse::<i32>().unwrap()),
        }
    }
}

impl From<&rspotify::model::ArtistId> for UniversalId {
    fn from(input : &rspotify::model::ArtistId) -> Self {
        UniversalId::Spotify(input.to_string())
    }
}

impl From<&rspotify::model::AlbumId> for UniversalId {
    fn from(input : &rspotify::model::AlbumId) -> Self {
        UniversalId::Spotify(input.to_string())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RequestPage {
    offset: Option<i64>,
    limit: Option<i64>,
}

impl RequestPage {
    pub fn new(offset: i64, limit: i64) -> Self {
        Self {
            offset: Some(Self::_offset(offset)),
            limit: Some(Self::_limit(limit)),
        }
    }

    pub fn offset(&self) -> i64 {
        match self.offset {
            Some(o) => Self::_offset(o),
            None => 0
        }
    }

    pub fn limit(&self) -> i64 {
        match self.limit {
            Some(l) => Self::_limit(l),
            None => 50
        }
    }

    pub fn next(&self) -> Self {
        Self::new(self.offset() + self.limit(), self.limit())
    }

    pub fn prev(&self) -> Self {
        Self::new(self.offset() - self.limit(), self.limit())
    }

    fn _offset(offset: i64) -> i64 {
        if offset < 0 { 0 } else { offset }
    }

    fn _limit(limit: i64) -> i64 {
        if !(0..=50).contains(&limit) { 50 } else { limit }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ResponsePage {
    offset: i32,
    limit: i32,
    next: Option<String>,
    prev: Option<String>,
}

impl ResponsePage {
    pub fn new(api: &str, page: &RequestPage, full_page: bool) -> Self {
        let next = if full_page { Some(Self::_api_string(api, &page.next())) } else { None };
        let prev = if page.offset() > 0 { Some(Self::_api_string(api, &page.prev())) } else { None };
        Self {
            offset: page.offset() as i32,
            limit: page.limit() as i32,
            next, prev,
        }
    }

    fn _api_string(api: &str, page: &RequestPage) -> String {
        let mut api = api.to_string();
        if api.contains('?') {
            api += "&offset=";
        } else {
            api += "?offset=";
        }
        api += &format!("{}", page.offset());
        api += "&limit=";
        api += &format!("{}", page.limit());
        api
    }
}

pub enum AlbumType {
    Single,
    Album,
    Compilation,
    AppearsOn,
}

impl From<i32> for AlbumType {
    fn from(t: i32) -> Self {
        match t {
            0 => AlbumType::Album,
            1 => AlbumType::Single,
            2 => AlbumType::Compilation,
            3 => AlbumType::AppearsOn,
            _ => panic!("Unknown album type!")
        }
    }
}

impl From<AlbumType> for i32 {
    fn from(t: AlbumType) -> Self {
        match t {
            AlbumType::Album => 0,
            AlbumType::Single => 1,
            AlbumType::Compilation => 2,
            AlbumType::AppearsOn => 3
        }
    }
}

impl From<rspotify::model::AlbumType> for AlbumType {
    fn from(t: rspotify::model::AlbumType) -> Self {
        match t {
            rspotify::model::AlbumType::Single => AlbumType::Single,
            rspotify::model::AlbumType::Album => AlbumType::Album,
            rspotify::model::AlbumType::Compilation => AlbumType::Compilation,
            rspotify::model::AlbumType::AppearsOn => AlbumType::AppearsOn
        }
    }
}