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
        match &input[..2] {
            "s:" => UniversalId::Spotify(input[2..].to_string()),
            "d:" => UniversalId::Database(input[2..].parse::<i32>().unwrap()),
            _ => panic!("Invalid UniversalId!")
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Page {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

impl Page {
    pub fn new(offset: i64, limit: i64) -> Self {
        Self {
            offset: Some(offset),
            limit: Some(limit),
        }
    }

    pub fn offset(&self) -> i64 {
        match self.offset {
            Some(o) => if o < 0 { 0 } else { o },
            None => 0
        }
    }

    pub fn limit(&self) -> i64 {
        match self.limit {
            Some(l) => if !(0..=50).contains(&l) { 50 } else { l },
            None => 50
        }
    }
}

pub enum AlbumType {
    Single,
    Album,
    Compilation,
    AppearsOn
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