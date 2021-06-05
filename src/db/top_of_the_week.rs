use super::{Result, DB, Save, db_error::DbError};
use super::util::last_row_id;
use super::song::Song;

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

pub trait TopOfTheWeek {
    fn get_current_top_of_week() -> Vec<TopOfTheWeekEntry>;
    fn get_top_of_week(year: u16, week: u8) -> Vec<TopOfTheWeekEntry>;
}

impl Save<TopOfTheWeekEntry> for DB {
    fn save(&mut self, to_save: &mut TopOfTheWeekEntry) -> Result<()> {
        debug_assert!(to_save.song_id != 0);

        if to_save.id == 0 {
            //Do insert
            let result: usize = {
                let mut stmt = self.prepare("INSERT INTO top_charts_of_week (calendar_week,year,source_name,song_id,song_position) VALUES(?,?,?,?,?)")?;
                stmt.execute(rusqlite::params![to_save.week, to_save.year, to_save.source, to_save.song_id, to_save.chart_position])?
            };
            if result == 1 {
                to_save.id = last_row_id(self)?;
                Ok(())
            }else{
                Err(DbError::new("Failed to ceate new top chart of the week entry with given data!"))
            }
        }else {
            //Do Update
            let mut stmt = self.prepare("UPDATE top_charts_of_week SET calendar_week = ?, year = ?, source_name = ?, song_id = ?, song_position = ? WHERE week_song_id = ?")?;
            let result = stmt.execute(rusqlite::params![to_save.week, to_save.year, to_save.source, to_save.song_id, to_save.chart_position, to_save.id])?;

            if result != 1 {
                Err(DbError::new("Failed to update top chart of week entry with given data!"))
            }else{
                Ok(())
            }
        }
    }
}