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