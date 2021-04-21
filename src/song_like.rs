use std::fmt::Formatter;
use serde::{Serialize, Deserialize};
use crate::song_like_protocol_generated;
use crate::error;
use crate::error::SoundbaseError;

#[derive(Debug)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub raw: String,
}

pub struct SongSource<'a> {
    pub source_kind: String,
    pub source_name: String,
    pub dissect_info: Option<&'a SourceMetadataDissect>,
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
pub struct SourceMetadataDissect {
    pub source_kind: String,
    pub source_name: String,
    pub dissect_regexp: String,
    pub mapping: Vec<SourceMetadataDissectMapping>,
    pub exclude: Vec<SourceMetadataDissectExclude>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceMetadataDissectExclude {
    pub matching_group: u8,
    pub if_equals: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceMetadataDissectMapping {
    pub matching_group: u8,
    pub field: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceMetadataDissectConfig {
    pub sources: Vec<SourceMetadataDissect>
}

impl Song {
    pub fn new(s: &song_like_protocol_generated::SongInfo) -> Self {
        Song {
            title: s.song_title().unwrap_or("").to_string(),
            artist: s.song_artist().unwrap_or("").to_string(),
            album: s.song_album().unwrap_or("").to_string(),
            raw: s.raw_meta().to_string(),
        }
    }

    pub fn has_only_raw(&self) -> bool {
        self.title.is_empty() && self.artist.is_empty() && self.album.is_empty()
    }

    pub fn dissect_raw_using_source_info(&mut self, source: &SongSource) -> error::Result<()> {
        let dissect = source.dissect_info.ok_or(SoundbaseError{http_code:tide::StatusCode::InternalServerError, msg: "No Dissect Info. Skipping.".to_string()})?;
        let rxp = &dissect.dissect_regexp;
        let regex = regex::Regex::new(rxp)?;
        let excludes = &dissect.exclude;
        let mappings = &dissect.mapping;

        let capture = regex.captures(&self.raw)
            .ok_or_else(|| error::SoundbaseError { http_code: tide::StatusCode::InternalServerError, msg: "Didn't match capturing group for dissect!".to_string() })?;


        let mut found_excludes = excludes.iter()
            .filter(|ex| capture.get(ex.matching_group as usize) != None)
            .filter(|ex| capture.get(ex.matching_group as usize).unwrap().as_str() == ex.if_equals.as_str());

        match found_excludes.next() {
            Some(ex) => {
                println!("\tExcluding song -> {:?} due to exclude -> {:?}", self, ex);
                Err(error::SoundbaseError { http_code: tide::StatusCode::InternalServerError, msg: "Found Excluded Song!".to_string() })
            }
            None => {
                let mut found_matches = mappings.iter().filter(|m| capture.get(m.matching_group as usize) != None);
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
                Ok(())
            }
        }
    }
}

impl<'a> SongSource<'a> {
    pub fn new(s: &'a song_like_protocol_generated::SongSourceInfo, dissects: &'a [SourceMetadataDissect]) -> Self {
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

fn try_get_fitting_dissect<'a>(dissects: &'a [SourceMetadataDissect], kind: &'a str, name: &'a str) -> std::option::Option<&'a SourceMetadataDissect> {
    let mut filtered = dissects.iter().filter(|e| {
        println!("\t\tChecking dissect => {:?}", e);
        println!("\t\tComparing {:?} == {:?} && {:?} == {:?}", e.source_kind, kind, e.source_name, name);
        e.source_kind == kind && e.source_name == name
    });

    match filtered.next() {
        Some(fit) => Some(fit),
        None => {
            println!("\tDidn't find fitting dissect; returning empty.");
            None
        }
    }
}

impl std::fmt::Display for Song {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "raw: {:?}; title: {:?}; artist: {:?}; album: {:?}", self.raw, self.title, self.artist, self.album)
    }
}

impl std::fmt::Display for SongSource<'_> {
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

impl<'a> SourceMetadataDissectConfig {
    pub fn load_from_file(path: &str) -> Self {
        let file = std::fs::File::open(path)
            .expect("Failed to open File!");
        let reader = std::io::BufReader::new(file);

        let s: SourceMetadataDissectConfig = serde_json::from_reader(reader)
            .expect("Failed to parse JSON to Datatype!");
        s
    }
}