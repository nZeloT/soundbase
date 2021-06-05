use super::{Result, DB};
use crate::model::analytics::{Metadata, PlaybackChange, SongChange, PageChange};

pub trait AnalyticsDB {
    fn store_page_change(&mut self, meta: &Metadata, change: &PageChange) -> Result<()>;
    fn store_playback_change(&mut self, meta: &Metadata, playback: &PlaybackChange) -> Result<()>;
    fn store_song_change(&mut self, meta: &Metadata, song: &SongChange) -> Result<()>;
}

impl AnalyticsDB for DB {
    fn store_page_change(&mut self, meta: &Metadata, change: &PageChange) -> Result<()> {
        let mut stmt = self.prepare_cached("INSERT INTO analytics (tmstp,origin,kind,transition_src,transition_dst) VALUES (?,?,?,?,?)")?;

        let kind: u8 = meta.kind.into();
        let src: u8 = change.src.into();
        let dst: u8 = change.dst.into();

        stmt.execute(rusqlite::params![
            meta.tmstp.timestamp_millis(),
            &meta.origin,
            kind,
            src,
            dst
        ])?;

        println!("\tPage Change stored.");
        Ok(())
    }

    fn store_playback_change(&mut self, meta: &Metadata, playback: &PlaybackChange) -> Result<()> {
        let mut stmt = self.prepare_cached("INSERT INTO analytics (tmstp,origin,kind,playback_source,playback_name,playback_started) VALUES (?,?,?,?,?,?)")?;
        let kind: u8 = meta.kind.into();
        let source: u8 = playback.source.into();
        stmt.execute(rusqlite::params![
            meta.tmstp.timestamp_millis(),
            &meta.origin,
            kind,
            source,
            &playback.name,
            playback.started
        ])?;
        println!("\tPlayback Change stored.");
        Ok(())
    }

    fn store_song_change(&mut self, meta: &Metadata, song: &SongChange) -> Result<()> {
        let mut stmt = self.prepare_cached("INSERT INTO analytics (tmstp,origin,kind,song_raw,song_title,song_artist,song_album) VALUES (?,?,?,?,?,?,?)")?;
        let kind: u8 = meta.kind.into();
        stmt.execute(rusqlite::params![
            meta.tmstp.timestamp_millis(),
            &meta.origin,
            kind,

            &song.raw_meta,
            &song.title,
            &song.artist,
            &song.album
        ])?;
        println!("\tSong Change stored.");
        Ok(())
    }
}