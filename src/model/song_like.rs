use std::fmt::Formatter;
use serde::{Serialize, Deserialize};
use crate::generated::song_like_protocol_generated as protocol;
use crate::error::{SoundbaseError, Result};

#[derive(Debug, Clone)]
pub struct RawSong {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub raw: String,
}

pub struct SongSource<'a> {
    pub source_kind: String,
    pub source_name: String,
    pub dissect_info: Option<&'a SourceMetadataDetermination>,
}

#[derive(Debug)]
pub enum SongState {
    NotFound,
    Faved,
    NotFaved,
    NowFaved,
    NowNotFaved,
}

pub struct SongMetadata<'a> {
    pub origin: String,
    pub source: SongSource<'a>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceMetadataDetermination {
    pub source_kind: String,
    pub source_name: Option<String>,
    pub dissect: Option<SourceMetadataDerive<DissectMapping>>,
    pub exclude: Option<SourceMetadataDerive<ExcludeMapping>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceMetadataDerive<T> {
    pub regexp: String,
    pub mapping: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExcludeMapping {
    pub matching_group: u8,
    pub if_equals: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DissectMapping {
    pub matching_group: u8,
    pub field: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceMetadataDeterminationConfig {
    pub sources: Vec<SourceMetadataDetermination>,
}

impl RawSong {
    pub fn new(s: &protocol::SongInfo) -> Self {
        RawSong {
            title: s.song_title().unwrap_or("").to_string(),
            artist: s.song_artist().unwrap_or("").to_string(),
            album: s.song_album().unwrap_or("").to_string(),
            raw: s.raw_meta().to_string(),
        }
    }

    pub fn has_only_raw(&self) -> bool {
        self.title.is_empty() && self.artist.is_empty() && self.album.is_empty()
    }

    pub fn dissect_raw_using_source_info(&mut self, source: &SongSource) -> Result<()> {
        let determination = source.dissect_info
            .ok_or(SoundbaseError { http_code: http::StatusCode::INTERNAL_SERVER_ERROR, msg: "No Dissect Info. Skipping.".to_string() })?;

        //check for excludes
        if let Some(ref ex) = determination.exclude {
            let regexp = regex::Regex::new(&ex.regexp)?;
            let capture = regexp.captures(&self.raw)
                .ok_or_else(|| SoundbaseError::new("Couldn't match exclude regexp for checking!"))?;
            let mut matches_exclude = ex.mapping.iter()
                .filter(|mapping| capture.get(mapping.matching_group as usize) != None)
                .filter(|mapping| capture.get(mapping.matching_group as usize).unwrap().as_str() == mapping.if_equals.as_str());

            if let Some(ex) = matches_exclude.next() {
                println!("\tExcluding song -> {:?} due to exclude -> {:?}", self, ex);
                return Err(SoundbaseError::new("Found Excluded Song!"))
            }
        }

        //determine values
        if let Some(ref dis) = determination.dissect {
            let regexp = regex::Regex::new(&dis.regexp)?;
            let capture = regexp.captures(&self.raw)
                .ok_or_else(|| SoundbaseError::new("Couldn't match dissect regexp for attribute mapping"))?;

            let found_matches = dis.mapping.iter().filter(|mapping| capture.get(mapping.matching_group as usize) != None);
            for mapping in found_matches {
                let m = capture.get(mapping.matching_group as usize).unwrap(); //is safe as all others have been filtered before
                let value = m.as_str();

                match mapping.field.to_uppercase().as_str() {
                    "TITLE" => self.title = value.to_string(),
                    "ARTIST" => self.artist = value.to_string(),
                    "ALBUM" => self.album = value.to_string(),
                    _ => {
                        println!("\tFound unknown mapping type named -> {}", mapping.field);
                    }
                }
            }
        }

        Ok(())
    }
}

impl<'a> SongSource<'a> {
    pub fn new(s: &'a protocol::SongSourceInfo, dissects: &'a [SourceMetadataDetermination]) -> Self {
        let kind = s.source_kind().variant_name().expect("Received unknown SourceKind!");
        let name = s.source_name();
        let dissect = try_get_fitting_dissect(dissects, kind, name);
        SongSource {
            source_kind: kind.to_string(),
            source_name: name.to_string(),
            dissect_info: dissect,
        }
    }
}

fn try_get_fitting_dissect<'a>(dissects: &'a [SourceMetadataDetermination], kind: &'a str, name: &'a str) -> std::option::Option<&'a SourceMetadataDetermination> {
    let mut filtered = dissects.iter().filter(|e| {
        println!("\t\tChecking dissect => {:?}", e);
        println!("\t\tComparing {:?} == {:?} && {:?} == {:?}", e.source_kind, kind, e.source_name, name);
        e.source_kind == kind && (e.source_name.is_none() || e.source_name.as_ref().unwrap().as_str() == name)
    });

    match filtered.next() {
        Some(fit) => Some(fit),
        None => {
            println!("\tDidn't find fitting dissect; returning empty.");
            None
        }
    }
}


impl std::fmt::Display for RawSong {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "raw: {:?}; title: {:?}; artist: {:?}; album: {:?}", self.raw, self.title, self.artist, self.album)
    }
}

impl std::fmt::Display for SongSource<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "kind: {}; name: {}", self.source_kind, self.source_name)
    }
}

impl Into<protocol::ResponseKind> for SongState {
    fn into(self) -> protocol::ResponseKind {
        match self {
            SongState::NotFound => protocol::ResponseKind::NOT_FOUND,
            SongState::Faved => protocol::ResponseKind::FOUND_FAVED,
            SongState::NotFaved => protocol::ResponseKind::FOUND_NOT_FAVED,
            SongState::NowFaved => protocol::ResponseKind::FOUND_NOW_FAVED,
            SongState::NowNotFaved => protocol::ResponseKind::FOUND_NOW_NOT_FAVED
        }
    }
}

impl<'a> SourceMetadataDeterminationConfig {
    pub fn load_from_file(path: &str) -> Self {
        let file = std::fs::File::open(path)
            .expect("Failed to open File!");
        let reader = std::io::BufReader::new(file);

        let s: SourceMetadataDeterminationConfig = serde_json::from_reader(reader)
            .expect("Failed to parse JSON to Datatype!");
        s
    }
}