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


use chrono::Datelike;
use super::{Result, DbPool, Load, Save, Delete, FollowForeignReference, db_error::DbError};
use super::util::{last_row_id, delete};
use super::album::Album;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AlbumOfTheWeekSource {
    Unkonwn,
    RockAntenne,
}

impl From<&str> for AlbumOfTheWeekSource {
    fn from(input: &str) -> Self {
        match input.to_uppercase().as_str() {
            "ROCKANTENNE" | "ROCK ANTENNE" => AlbumOfTheWeekSource::RockAntenne,
            _ => AlbumOfTheWeekSource::Unkonwn
        }
    }
}

impl AlbumOfTheWeekSource {
    pub fn to_string(&self) -> String {
        match self {
            AlbumOfTheWeekSource::Unkonwn => "Unknown".to_string(),
            AlbumOfTheWeekSource::RockAntenne => "Rock Antenne".to_string()
        }
    }
}

#[derive(Debug, Eq)]
pub struct AlbumOfTheWeek {
    id: u64,
    album_id: u64,
    album_song_list_raw: String,
    source: AlbumOfTheWeekSource,
    source_comment: String,
    source_date: chrono::DateTime<chrono::FixedOffset>,
    year: u64,
    week: u8,
}

impl AlbumOfTheWeek {
    //TODO: return Result and check whether album id is valid
    pub fn new(source: String, source_comment: String, source_date: chrono::DateTime<chrono::FixedOffset>, album: Album, album_song_list_raw: String) -> Self {
        AlbumOfTheWeek {
            id: 0,
            source: AlbumOfTheWeekSource::from(source.as_str()),
            source_comment,
            source_date,
            album_song_list_raw,
            album_id: album.id,
            year: source_date.year() as u64,
            week: source_date.iso_week().week() as u8,
        }
    }
}

impl PartialEq for AlbumOfTheWeek {
    fn eq(&self, other: &Self) -> bool {
        println!("Comparing two AofW...");
        println!("\tsource => {:?} <=> {:?} => {:?}", self.source, other.source, self.source == other.source);
        println!("\tdate => {:?} <=> {:?} => {:?}", self.source_date, other.source_date, self.source_date == other.source_date);
        println!("\talbum => {:?} <=> {:?} => {:?}", self.album_id, other.album_id, self.album_id == other.album_id);
        self.source == other.source && self.source_date == other.source_date && self.album_id == other.album_id
    }
}

pub trait AlbumsOfTheWeek {
    fn get_current_album_of_week(&self, source: AlbumOfTheWeekSource) -> Result<Option<AlbumOfTheWeek>>;
    fn get_albums_of_week(&self, offset: u8, limit: u8, source: AlbumOfTheWeekSource, year: Option<u64>, week: Option<u64>) -> Result<Vec<AlbumOfTheWeek>>;
}

impl AlbumsOfTheWeek for DbPool {
    fn get_current_album_of_week(&self, source: AlbumOfTheWeekSource) -> Result<Option<AlbumOfTheWeek>> {
        let mut albums = self.get_albums_of_week(0, 1, source, None, None)?;
        Ok(albums.pop())
    }

    fn get_albums_of_week(&self, offset: u8, limit: u8, source: AlbumOfTheWeekSource, year: Option<u64>, week: Option<u64>) -> Result<Vec<AlbumOfTheWeek>> {
        let query: String = {
            let mut base: String = "SELECT * FROM albums_of_week WHERE source_name = ?".to_string();
            if year.is_some() || week.is_some() {
                if let Some(y) = year {
                    base += " AND year = ";
                    base += &*y.to_string();
                }

                if let Some(w) = week {
                    base += " AND week = ";
                    base += &*w.to_string();
                }
            }

            base += "order by year desc, week desc";
            base += " LIMIT ? OFFSET ?";
            base
        };
        let albums = match self.get() {
            Ok(mut conn) => {
                let mut stmt = conn.prepare(query.as_str())?;
                let mut rows = stmt.query(rusqlite::params![source.to_string(), limit, offset])?;
                let mut result = Vec::<AlbumOfTheWeek>::new();
                while let Some(row) = rows.next()? {
                    let dt_str: String = row.get(5)?;
                    let tmstp = chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&dt_str)?;
                    result.push(AlbumOfTheWeek {
                        id: row.get(0)?,
                        album_id: row.get(1)?,
                        album_song_list_raw: row.get(2)?,
                        source: AlbumOfTheWeekSource::from(row.get::<usize, String>(3)?.as_str()),
                        source_comment: row.get(4)?,
                        source_date: tmstp,
                        year: row.get(6)?,
                        week: row.get(7)?,
                    });
                }
                Ok(result)
            }
            Err(_) => Err(DbError::pool_timeout())
        }?;
        Ok(albums)
    }
}

impl Load<AlbumOfTheWeek> for DbPool {
    fn load(&self, id: u64) -> Result<AlbumOfTheWeek> {
        if id == 0 {
            Err(DbError::new("Invaild ID given!"))
        } else {
            match self.get() {
                Ok(mut conn) => {
                    let mut stmt = conn.prepare("SELECT album_id,album_song_list_raw,source_name,source_comment,source_date,year,week FROM albums_of_week WHERE aow_id = ? LIMIT 1")?;
                    let mut rows = stmt.query(rusqlite::params![id])?;

                    match rows.next()? {
                        Some(row) => {
                            let tmstp: String = row.get(4)?;
                            let d = chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&tmstp)?;

                            Ok(AlbumOfTheWeek {
                                id,
                                album_id: row.get(0)?,
                                album_song_list_raw: row.get(1)?,
                                source: AlbumOfTheWeekSource::from(row.get::<usize, String>(2)?.as_str()),
                                source_comment: row.get(3)?,
                                source_date: d,
                                year: row.get(5)?,
                                week: row.get(6)?,
                            })
                        }
                        None => Err(DbError::new("Didn't find the album of the week for the given week id!"))
                    }
                }
                Err(_) => Err(DbError::pool_timeout())
            }
        }
    }
}

impl FollowForeignReference<AlbumOfTheWeek, Album> for DbPool {
    fn follow_reference(&self, to_follow: &AlbumOfTheWeek) -> Result<Album> {
        self.load(to_follow.album_id)
    }
}

impl Save<AlbumOfTheWeek> for DbPool {
    fn save(&self, to_save: &mut AlbumOfTheWeek) -> Result<()> {
        debug_assert!(to_save.album_id != 0);
        let date_time = to_save.source_date.to_rfc3339();

        match self.get() {
            Ok(mut conn) => {
                if to_save.id == 0 {
                    //do insert
                    let result: usize = {
                        let mut stmt = conn.prepare("INSERT INTO albums_of_week (album_id,album_song_list_raw,source_name,source_comment,source_date,year,week) VALUES(?,?,?,?,?,?,?)")?;
                        stmt.execute(rusqlite::params![to_save.album_id, to_save.album_song_list_raw, to_save.source.to_string(), to_save.source_comment, date_time, to_save.year, to_save.week])?
                    };
                    if result == 1 {
                        to_save.id = last_row_id(&mut conn)?;
                        Ok(())
                    } else {
                        Err(DbError::new("Failed to create new album of week entry with given data!"))
                    }
                } else {
                    //Do update
                    let mut stmt = conn.prepare("UPDATE albums_of_week SET album_id = ?, album_song_list_raw = ?, source_name = ?, source_comment = ?, source_date = ?, year = ?, week = ? WHERE aow_id = ?")?;
                    let result = stmt.execute(rusqlite::params![to_save.album_id, to_save.album_song_list_raw, to_save.source.to_string(), to_save.source_comment, date_time, to_save.year, to_save.week, to_save.id])?;

                    if result != 1 {
                        Err(DbError::new("Failed to update the given album of week!"))
                    } else {
                        Ok(())
                    }
                }
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}

impl Delete<AlbumOfTheWeek> for DbPool {
    fn delete(&self, to_delete: &AlbumOfTheWeek) -> Result<()> {
        match self.get() {
            Ok(mut conn) => {
                delete(&mut conn, "albums_of_week", "week_id", to_delete.id)
            }
            Err(_) => Err(DbError::pool_timeout())
        }
    }
}