use super::{Result, DB};
use super::song::Song;

use crate::model::song_like::SongMetadata;

pub trait SongHistDB {
    fn store_song_hist_entry(&mut self, song: &Song, meta: &SongMetadata, new_state: bool) -> Result<()>;
}

impl SongHistDB for DB {
    fn store_song_hist_entry(&mut self, song: &Song, meta: &SongMetadata, new_state: bool) -> Result<()> {
        println!("\tStored in Hist DB!");

        let mut stmt = self.prepare("INSERT INTO song_history (song_id,change_origin,change_source,change_name,prev_state,new_state) VALUES(?,?,?,?,?,?)")?;
        stmt.execute(rusqlite::params![
            song.id,
            &meta.origin,
            &meta.source.source_kind,
            &meta.source.source_name,
            !new_state,
            new_state
        ])?;

        Ok(())
    }
}