use std::fmt::Formatter;
use crate::song_like_protocol_generated;

pub struct Song {
    pub title:  String,
    pub artist: String,
    pub album:  String,
    pub raw:    String
}

pub struct SongSource {
    pub source_kind: String,
    pub source_name: String
}

pub enum SongState {
    NotFound,
    Faved,
    NotFaved,
    NowFaved,
    NowNotFaved
}

pub struct SongMetadata {
    pub origin: String,
    pub source: SongSource
}

impl Song {
    pub fn new(s: &song_like_protocol_generated::SongInfo) -> Self {
        Song {
            title: s.song_title().unwrap_or("").to_string(),
            artist: s.song_artist().unwrap_or("").to_string(),
            album: s.song_album().unwrap_or("").to_string(),
            raw: s.raw_meta().to_string()
        }
    }

    pub fn has_only_raw(&self) -> bool {
        self.title.is_empty() && self.artist.is_empty() && self.album.is_empty()
    }

    pub fn dissect_raw_using_source_info(&mut self, source : &SongSource) {
        todo!()
    }
}

impl SongSource {
    pub fn new(s: &song_like_protocol_generated::SongSourceInfo) -> Self {
        SongSource {
            source_kind: stringify!(s.source_kind()).into(),
            source_name: s.source_name().to_string()
        }
    }
}

impl std::fmt::Display for Song {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "raw: {:?}; title: {:?}; artist: {:?}; album: {:?}", self.raw, self.title, self.artist, self.album)
    }
}

impl std::fmt::Display for SongSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "kind: {}; name: {}", self.source_kind, self.source_name)
    }
}

impl Into<song_like_protocol_generated::ResponseKind> for SongState {
    fn into(self) -> song_like_protocol_generated::ResponseKind {
        match self {
            SongState::NotFound => song_like_protocol_generated::ResponseKind::NOT_FOUND,
            SongState::Faved => song_like_protocol_generated::ResponseKind::FOUND_FAVED,
            SongState::NotFaved => song_like_protocol_generated::ResponseKind::FOUND_NOT_FAVED,
            SongState::NowFaved => song_like_protocol_generated::ResponseKind::FOUND_NOW_FAVED,
            SongState::NowNotFaved => song_like_protocol_generated::ResponseKind::FOUND_NOW_NOT_FAVED
        }
    }
}