/*
 * Copyright 2022 nzelot<leontsteiner@gmail.com>
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

use crate::db_new::DbApi;
use crate::db_new::models::{NewTrackFavProposal, TrackFavProposal};
use crate::db_new::track::TrackDb;
use crate::db_new::track_fav_proposal::TrackFavProposalDb;
use crate::error::SoundbaseError;
use super::reply;
use crate::generated::song_like_protocol_generated as protocol;

trait RawResolver {
    fn is_excluded(&self, raw: &str) -> bool;
    fn track(&self, raw: &str) -> String;
    fn album(&self, raw: &str) -> Option<String>;
    fn artist(&self, raw: &str) -> String;
}

pub async fn handle_song_like_message(db: DbApi, body: bytes::Bytes) -> Result<impl warp::Reply, std::convert::Infallible> {
    let resp = process_message(&db, body.to_vec());

    match resp {
        Ok(r) => Ok(reply(r, http::StatusCode::OK)),
        Err(e) => {
            println!("\tResponding with Error => {:?}", e.msg);
            Ok(reply(e.msg.as_bytes().to_vec(), e.http_code))
        }
    }
}

fn process_message(db: &DbApi, buffer: Vec<u8>) -> Result<Vec<u8>, SoundbaseError>
{
    let msg = protocol::root_as_song_message(buffer.as_slice())
        .expect("Expected SongMessage. Got something else!");

    print!("Received Song Like Message with id {:?} ", msg.id());

    assert_eq!(msg.payload_type(), protocol::MessagePayload::Request);

    let request = msg.payload_as_request().unwrap();
    println!("From requesting party {:?}", request.requesting_party());
    let song_info = &request.song_info();
    let song_source = &request.song_source_info();

    let source_name = get_source_name(song_source);
    let raw = song_info.raw_meta();

    let resolver = get_resolver(&*source_name);
    let resp_kind = if resolver.is_excluded(raw) {
        protocol::ResponseKind::NOT_FOUND
    } else {
        let proposal = find_on_db(db, &*source_name, raw)?;
        match request.action_kind()
        {
            protocol::RequestAction::FAV => {
                //Add new TrackFavProposal
                process_fav(db, &resolver, proposal, &*source_name, raw)?
            }
            protocol::RequestAction::UNFAV => {
                //Change TrackFavProposal state / and possibly song state
                process_unfav(db, proposal)?
            }
            protocol::RequestAction::INFO => {
                //Match with fav proposal
                process_info(proposal)?
            }
            _ => return Err(SoundbaseError::new("Unknown Request Kind!")),
        }
    };

    //now build response
    build_response_message(msg.id(), resp_kind)
}

fn process_fav<DB>(db: &DB, resolver : &Box<dyn RawResolver>, proposal : Option<TrackFavProposal>, source_name : &str, raw : &str) -> Result<protocol::ResponseKind, SoundbaseError>
where DB : TrackFavProposalDb {
    match proposal {
        Some(_) => Ok(protocol::ResponseKind::FOUND_NOW_FAVED),
        None => {
            //create a new proposal
            db.new_track_proposal(get_new_track_proposal(resolver, source_name, raw))?;
            Ok(protocol::ResponseKind::FOUND_NOW_FAVED)
        }
    }
}

fn process_unfav<DB>(db: &DB, proposal : Option<TrackFavProposal>) -> Result<protocol::ResponseKind, SoundbaseError>
    where DB: TrackDb + TrackFavProposalDb {
    match proposal {
        Some(p) => {
            //delete proposal and set track to not faved if faved
            set_track_to_unfaved(db, &p.track_id)?;
            db.delete_track_proposal(p.track_fav_id)?;
            Ok(protocol::ResponseKind::FOUND_NOW_NOT_FAVED)
        }
        None => Ok(protocol::ResponseKind::FOUND_NOW_NOT_FAVED)
    }
}

fn process_info(proposal : Option<TrackFavProposal>) -> Result<protocol::ResponseKind, SoundbaseError> {
    match proposal {
        Some(_) => Ok(protocol::ResponseKind::FOUND_FAVED),
        None => Ok(protocol::ResponseKind::FOUND_NOT_FAVED)
    }
}

fn get_source_name(source_info: &protocol::SongSourceInfo) -> String {
    match source_info.source_kind() {
        protocol::SourceKind::RADIO => source_info.source_name().to_string(),
        protocol::SourceKind::BLUETOOTH => "Bluetooth".to_string(),
        protocol::SourceKind::SNAPCAST => "Snapcast".to_string(),
        _ => panic!("Don't go here!")
    }
}

fn get_new_track_proposal(resolver : &Box<dyn RawResolver>, source_name : &str, raw : &str) -> NewTrackFavProposal {
    let ext_artist = resolver.artist(raw);
    let ext_track = resolver.track(raw);
    let ext_album = resolver.album(raw);

    NewTrackFavProposal {
        source_name: source_name.to_string(),
        source_prop: raw.to_string(),
        ext_track_title: ext_track,
        ext_artist_name: ext_artist,
        ext_album_name: ext_album,
    }
}

fn find_on_db<DB>(db: &DB, source_name : &str, pattern : &str) -> Result<Option<TrackFavProposal>, SoundbaseError>
    where DB: TrackFavProposalDb {
    Ok(db.find_by_source_and_raw_pattern(source_name, pattern)?)
}

fn set_track_to_unfaved<DB>(db: &DB, track_id: &Option<i32>) -> Result<(), SoundbaseError>
    where DB: TrackDb {
    match track_id {
        Some(id) => {
            db.set_faved_state(*id, false)?;
            Ok(())
        }
        None => Ok(())
    }
}

fn build_response_message(msg_id: u64, response: protocol::ResponseKind) -> Result<Vec<u8>, SoundbaseError> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();

    let mut res_builder = protocol::ResponseBuilder::new(&mut fbb);
    res_builder.add_kind(response);
    let resp = res_builder.finish();

    let mut resp_msg_builder = protocol::SongMessageBuilder::new(&mut fbb);
    resp_msg_builder.add_id(msg_id);
    resp_msg_builder.add_payload_type(protocol::MessagePayload::Response);
    resp_msg_builder.add_payload(resp.as_union_value());
    let resp_msg = resp_msg_builder.finish();

    println!();
    protocol::finish_song_message_buffer(&mut fbb, resp_msg);
    Ok(fbb.finished_data().to_vec())
}

fn get_resolver(source_name: &str) -> Box<dyn RawResolver> {
    match source_name {
        "Rock Antenne" => Box::new(RockAntenneResolver()),
        _ => Box::new(FullResolver())
    }
}

struct RockAntenneResolver();

struct FullResolver();

impl RawResolver for RockAntenneResolver {
    fn is_excluded(&self, raw: &str) -> bool {
        raw.starts_with("ROCK ANTENNE")
    }

    fn track(&self, raw: &str) -> String {
        raw.split('-').collect::<Vec<&str>>()[1].trim().to_string()
    }

    fn album(&self, _: &str) -> Option<String> {
        None
    }

    fn artist(&self, raw: &str) -> String {
        raw.split('-').collect::<Vec<&str>>()[0].trim().to_string()
    }
}

impl RawResolver for FullResolver {
    fn is_excluded(&self, _: &str) -> bool {
        false
    }

    //format is "Title - Arist (Album)"
    fn track(&self, raw: &str) -> String {
        raw.split('-').collect::<Vec<&str>>()[0].trim().to_string()
    }

    fn album(&self, raw: &str) -> Option<String> {
        let artist_album = raw.split('-').collect::<Vec<&str>>()[1].trim().to_string();
        let album = artist_album.split('(').collect::<Vec<&str>>()[1].trim().to_string();
        Some(album[..album.len() - 1].to_string())
    }

    fn artist(&self, raw: &str) -> String {
        let artist_album = raw.split('-').collect::<Vec<&str>>()[1].trim().to_string();
        artist_album.split('(').collect::<Vec<&str>>()[0].trim().to_string()
    }
}