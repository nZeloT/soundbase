use crate::error::{SoundbaseError, Result};
use crate::song_db::{SongDB, FindUnique, Save};
use crate::song_like::{SongState, SourceMetadataDissect, RawSong, SongSource, SongMetadata, SongFav};
use crate::song_like_protocol_generated;

pub fn consume_like_message(db: &mut SongDB, dissects: &[SourceMetadataDissect], buffer: Vec<u8>) -> Result<Vec<u8>>
{
    let msg = song_like_protocol_generated::root_as_song_message(buffer.as_slice())
        .expect("Expected SongMessage. Got something else!");

    print!("Received Song Like Message with id {:?} ", msg.id());

    assert_eq!(msg.payload_type(), song_like_protocol_generated::MessagePayload::Request);

    let resp_kind = process_message(db, dissects, &msg.payload_as_request().unwrap())?;

    //now build response
    build_response_message(msg.id(), resp_kind)
}

fn build_response_message(msg_id: u64, response: song_like_protocol_generated::ResponseKind) -> Result<Vec<u8>> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();

    let mut res_builder = song_like_protocol_generated::ResponseBuilder::new(&mut fbb);
    res_builder.add_kind(response);
    let resp = res_builder.finish();

    let mut resp_msg_builder = song_like_protocol_generated::SongMessageBuilder::new(&mut fbb);
    resp_msg_builder.add_id(msg_id);
    resp_msg_builder.add_payload_type(song_like_protocol_generated::MessagePayload::Response);
    resp_msg_builder.add_payload(resp.as_union_value());
    let resp_msg = resp_msg_builder.finish();

    println!();
    song_like_protocol_generated::finish_song_message_buffer(&mut fbb, resp_msg);
    Ok(fbb.finished_data().to_vec())
}

fn process_message(db: &mut SongDB, dissects: &[SourceMetadataDissect], request: &song_like_protocol_generated::Request) -> Result<song_like_protocol_generated::ResponseKind>
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

    let possible_song = db.find_unique(&song_info)?;

    let resp_kind = match request.action_kind() {
        song_like_protocol_generated::RequestAction::INFO => {
            match possible_song {
                Some(song) => if song.is_faved { Ok(SongState::Faved) } else { Ok(SongState::NotFaved) },
                None => Ok(SongState::NotFound)
            }
        },
        song_like_protocol_generated::RequestAction::FAV => {
            //load song
            match possible_song {
                Some(mut song) => db.fav_song(&mut song, &song_meta),
                None => {
                    //song does not yet exist, so create it
                    let mut song = db.create_song_from_raw(&song_info)?;
                    song.is_faved = true;
                    db.save(&mut song)?;
                    Ok(SongState::NowFaved)
                }
            }
        },
        song_like_protocol_generated::RequestAction::UNFAV => {
            match possible_song {
                Some(mut song) => db.unfav_song(&mut song, &song_meta),
                None => {
                    //cant unfav a unknown song, therefore yield an error here
                    Err(SoundbaseError::new("Can't unfav unknown song!"))
                }
            }
        },
        _ => Err(SoundbaseError::new("Unknown RequestAction received!"))
    };
    match resp_kind {
        Ok(kind) => {
            println!("\tReturning song state => {:?}", kind);
            Ok(kind.into())
        },
        Err(e) => Err(SoundbaseError::from(e))
    }
}