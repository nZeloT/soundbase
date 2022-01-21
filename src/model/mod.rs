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

//pub mod spotify;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum UniversalTrackId {
    Spotify(String),
    Database(i32),
}

impl From<&str> for UniversalTrackId {
    fn from(input: &str) -> Self {
        match &input[..2] {
            "s:" => UniversalTrackId::Spotify(input[2..].to_string()),
            "d:" => UniversalTrackId::Database(input[2..].parse::<i32>().unwrap()),
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