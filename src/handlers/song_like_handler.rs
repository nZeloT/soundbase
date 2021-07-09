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

use crate::error::{SoundbaseError, Result};
use crate::model::song_like::{SongState, SourceMetadataDetermination, RawSong, SongSource, SongMetadata};
use crate::generated::song_like_protocol_generated as protocol;

type Spotify = std::sync::Arc<tokio::sync::RwLock<crate::model::spotify::Spotify>>;

pub async fn consume_like_message<'a>(spotify: Spotify, dissects: &[SourceMetadataDetermination], buffer: Vec<u8>) -> Result<Vec<u8>>
{
    let msg = protocol::root_as_song_message(buffer.as_slice())
        .expect("Expected SongMessage. Got something else!");

    print!("Received Song Like Message with id {:?} ", msg.id());

    assert_eq!(msg.payload_type(), protocol::MessagePayload::Request);

    let resp_kind = process_message(spotify, dissects, &msg.payload_as_request().unwrap()).await?;

    //now build response
    build_response_message(msg.id(), resp_kind)
}

fn build_response_message(msg_id: u64, response: protocol::ResponseKind) -> Result<Vec<u8>> {
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

async fn process_message<'a>(spotify: Spotify, dissects: &[SourceMetadataDetermination], request: &'_ protocol::Request<'_>) -> Result<protocol::ResponseKind>
{
    println!("from requesting party {:?}", request.requesting_party());

    let mut song_info = RawSong::new(&request.song_info());
    let song_source_info = request.song_source_info();
    let song_source = SongSource::new(&song_source_info, dissects);
    let song_meta = SongMetadata {
        origin: request.requesting_party().to_string(),
        source: song_source,
    };

    println!("\tWith given song source: {} and details: {}", &song_meta.source, song_info);
    println!("\tRequesting action: {:?}", request.action_kind());

    if song_info.has_only_raw() {
        song_info.dissect_raw_using_source_info(&song_meta.source)?;
        println!("\tDissected Metadata to => {:?}", song_info);
    }

    let possible_song = {
        spotify.read().await.find_song_in_library(&song_info).await
    };

    let resp_kind = match request.action_kind() {
        protocol::RequestAction::INFO => {
            match possible_song {
                Some(song) => if song.in_library { Ok(SongState::Faved) } else { Ok(SongState::NotFaved) },
                None => Ok(SongState::NotFound)
            }
        }
        protocol::RequestAction::FAV => {
            //load song
            match possible_song {
                Some(song) => {
                    match spotify.read().await.publish_song_like(&song).await {
                        Ok(_) => Ok(SongState::NowFaved),
                        Err(e) => Err(e)
                    }
                }
                None => Ok(SongState::NotFound)
            }
        }
        protocol::RequestAction::UNFAV => {
            match possible_song {
                Some(song) => {
                    match spotify.read().await.publish_song_dislike(&song).await {
                        Ok(_) => Ok(SongState::NowNotFaved),
                        Err(e) => Err(e)
                    }
                }
                None => Ok(SongState::NotFound)
            }
        }
        _ => Err(SoundbaseError::new("Unknown RequestAction received!"))
    };
    match resp_kind {
        Ok(kind) => {
            println!("\tReturning song state => {:?}", kind);
            Ok(kind.into())
        }
        Err(e) => Err(SoundbaseError::from(e))
    }
}