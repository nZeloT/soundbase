use chrono::{Utc, DateTime, TimeZone};
use crate::analytics_protocol_generated;

#[derive(Copy,Clone,Debug)]
pub enum Pages {
    Inactive,
    MenuSelection,
    StreamPlaying,
    StreamSelection,
    BluetoothPlaying,
    SnapcastPlaying,
    Options
}

#[derive(Copy,Clone,Debug)]
pub enum PlaybackSource {
    Stream,
    Bluetooth,
    Snapcast
}

#[derive(Copy,Clone,Debug)]
pub enum MessageKind {
    PageChange,
    PlaybackChange,
    SongChange
}

pub struct Metadata {
    pub tmstp: DateTime<Utc>,
    pub origin: String,
    pub kind:   MessageKind
}

pub struct PageChange {
    pub src: Pages,
    pub dst: Pages
}

pub struct PlaybackChange {
    pub source: PlaybackSource,
    pub name:   String,
    pub started:bool
}

pub struct SongChange {
    pub raw_meta:   String,
    pub title:      String,
    pub artist:     String,
    pub album:      String
}

impl Into<Pages> for analytics_protocol_generated::Page {
    fn into(self) -> Pages {
        match self {
            analytics_protocol_generated::Page::INACTIVE => Pages::Inactive,
            analytics_protocol_generated::Page::MENU_SELECTION => Pages::MenuSelection,
            analytics_protocol_generated::Page::RADIO_PLAYING => Pages::StreamPlaying,
            analytics_protocol_generated::Page::RADIO_SELECTION => Pages::StreamSelection,
            analytics_protocol_generated::Page::BT_PLAYING => Pages::BluetoothPlaying,
            analytics_protocol_generated::Page::SNAPCAST_PLAYING => Pages::SnapcastPlaying,
            analytics_protocol_generated::Page::OPTIONS => Pages::Options,
            _ => panic!("Found unknown Page!")
        }
    }
}

impl Into<u8> for Pages {
    fn into(self) -> u8 {
        match self {
            Pages::Inactive => 0,
            Pages::MenuSelection => 1,
            Pages::StreamPlaying => 2,
            Pages::StreamSelection => 3,
            Pages::BluetoothPlaying => 4,
            Pages::SnapcastPlaying => 5,
            Pages::Options => 9
        }
    }
}

impl Into<PlaybackSource> for analytics_protocol_generated::PlaybackSource {
    fn into(self) -> PlaybackSource {
        match self {
            analytics_protocol_generated::PlaybackSource::RADIO => PlaybackSource::Stream,
            analytics_protocol_generated::PlaybackSource::BLUETOOTH => PlaybackSource::Bluetooth,
            analytics_protocol_generated::PlaybackSource::SNAPCAST => PlaybackSource::Snapcast,
            _ => panic!("Found unknown Playback Source!")
        }
    }
}

impl Into<u8> for PlaybackSource {
    fn into(self) -> u8 {
        match self {
            PlaybackSource::Stream => 0,
            PlaybackSource::Bluetooth => 1,
            PlaybackSource::Snapcast => 2
        }
    }
}

impl Into<MessageKind> for analytics_protocol_generated::AnalyticsMessageType {
    fn into(self) -> MessageKind {
        match self {
            analytics_protocol_generated::AnalyticsMessageType::PageChange => MessageKind::PageChange,
            analytics_protocol_generated::AnalyticsMessageType::PlaybackChange => MessageKind::PlaybackChange,
            analytics_protocol_generated::AnalyticsMessageType::PlaybackSongChange => MessageKind::SongChange,
            _ => {
                panic!("Failed to map AnalyticsMessageType to MessageKind!");
            }
        }
    }
}

impl Into<u8> for MessageKind {
    fn into(self) -> u8 {
        match self {
            MessageKind::PageChange => 0,
            MessageKind::PlaybackChange => 1,
            MessageKind::SongChange => 2
        }
    }
}

impl Metadata {
    pub fn new(m:&analytics_protocol_generated::AnalyticsMessage) -> Self {
        Metadata {
            tmstp: Utc.timestamp((m.timestamp() / 1000) as i64, ((m.timestamp() % 1000) * 1000000) as u32),
            kind:  m.payload_type().into(),
            origin: m.origin().to_string()
        }
    }
}

impl PageChange {
    pub fn new(a :&analytics_protocol_generated::PageChange) -> Self {
        PageChange {
            src: a.origin().into(),
            dst: a.destination().into()
        }
    }
}

impl PlaybackChange {
    pub fn new(a :&analytics_protocol_generated::PlaybackChange) -> Self {
        PlaybackChange {
            source: a.source().into(),
            name: a.name().to_string(),
            started: a.started()
        }
    }
}

impl SongChange {
     pub fn new(a :&analytics_protocol_generated::PlaybackSongChange) -> Self {
         SongChange {
             raw_meta: a.raw_meta().to_string(),
             title: a.title().unwrap_or("").to_string(),
             artist: a.artist().unwrap_or("").to_string(),
             album: a.album().unwrap_or("").to_string()
         }
     }
}