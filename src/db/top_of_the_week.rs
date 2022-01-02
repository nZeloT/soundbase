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

use super::{db_error::DbError, DbPool, Result, Save};
use super::song::Song;
use super::util::last_row_id;

#[derive(Debug)]
pub struct TopOfTheWeekEntry {
    id: u64,
    pub week: u8,
    pub year: u16,
    pub source: String,
    song_id: u64,
    pub chart_position: u8
}

impl TopOfTheWeekEntry {
    //TODO: return result and check whether song id is valid
    pub fn new(year: u16, week: u8, source: String, position: u8, song: Song) -> Self {
        TopOfTheWeekEntry {
            id: 0,
            year,
            week,
            source,
            chart_position: position,
            song_id: song.id
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TopOfTheWeekSource {
    Unkonwn,
    RockAntenne,
}

impl From<&str> for TopOfTheWeekSource {
    fn from(input: &str) -> Self {
        match input.to_uppercase().as_str() {
            "ROCKANTENNE" | "ROCK ANTENNE" => TopOfTheWeekSource::RockAntenne,
            _ => TopOfTheWeekSource::Unkonwn
        }
    }
}

impl TopOfTheWeekSource {
    pub fn to_string(&self) -> String {
        match self {
            TopOfTheWeekSource::Unkonwn => "Unknown".to_string(),
            TopOfTheWeekSource::RockAntenne => "Rock Antenne".to_string()
        }
    }
}

pub trait TopOfTheWeek {
    fn get_current_top_of_week(&self, source : TopOfTheWeekSource) -> Result<Vec<TopOfTheWeekEntry>>;
    fn get_top_of_week(&self, offset: u8, limit : u8, source : TopOfTheWeekSource, year: Option<u16>, week: Option<u8>) -> Result<Vec<TopOfTheWeekEntry>>;
}

impl Save<TopOfTheWeekEntry> for DbPool {
    fn save(&self, to_save: &mut TopOfTheWeekEntry) -> Result<()> {
        debug_assert!(to_save.song_id != 0);

        match self.get() {
            Ok(mut conn) => {
                if to_save.id == 0 {
                    //Do insert
                    let result: usize = {
                        let mut stmt = conn.prepare("INSERT INTO top_charts_of_week (calendar_week,year,source_name,song_id,song_position) VALUES(?,?,?,?,?)")?;
                        stmt.execute(rusqlite::params![to_save.week, to_save.year, to_save.source, to_save.song_id, to_save.chart_position])?
                    };
                    if result == 1 {
                        to_save.id = last_row_id(&mut conn)?;
                        Ok(())
                    }else{
                        Err(DbError::new("Failed to create new top chart of the week entry with given data!"))
                    }
                }else {
                    //Do Update
                    let mut stmt = conn.prepare("UPDATE top_charts_of_week SET calendar_week = ?, year = ?, source_name = ?, song_id = ?, song_position = ? WHERE week_song_id = ?")?;
                    let result = stmt.execute(rusqlite::params![to_save.week, to_save.year, to_save.source, to_save.song_id, to_save.chart_position, to_save.id])?;

                    if result != 1 {
                        Err(DbError::new("Failed to update top chart of week entry with given data!"))
                    }else{
                        Ok(())
                    }
                }
            },
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}