use crate::error::{SoundbaseError, Result};
use crate::model::song_like::{SongState, SourceMetadataDissect, RawSong, SongSource, SongMetadata};
use crate::generated::song_like_protocol_generated as protocol;
use crate::db::{song::SongFav, FindUnique, song::Song, song::create_song_from_raw, Save, album::*, artist::*, db_error::DbError};

pub fn consume_like_message<'a, DB>(db: &mut DB, dissects: &[SourceMetadataDissect], buffer: Vec<u8>) -> Result<Vec<u8>>
    where DB: SongFav + FindUnique<Song, RawSong> + FindUnique<Artist, FindArtist> + FindUnique<Album, FindAlbum> + Save<Artist> + Save<Album> + Save<Song>
{
    let msg = protocol::root_as_song_message(buffer.as_slice())
        .expect("Expected SongMessage. Got something else!");

    print!("Received Song Like Message with id {:?} ", msg.id());

    assert_eq!(msg.payload_type(), protocol::MessagePayload::Request);

    let resp_kind = process_message(db, dissects, &msg.payload_as_request().unwrap())?;

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

fn process_message<'a, DB>(db: &mut DB, dissects: &[SourceMetadataDissect], request: &protocol::Request) -> Result<protocol::ResponseKind>
    where DB: SongFav + FindUnique<Song, RawSong> + FindUnique<Artist, FindArtist> + FindUnique<Album, FindAlbum> + Save<Artist> + Save<Album> + Save<Song>
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
        db.find_unique(song_info.clone())?
    };

    let resp_kind = match request.action_kind() {
        protocol::RequestAction::INFO => {
            match possible_song {
                Some(song) => if song.is_faved { Ok(SongState::Faved) } else { Ok(SongState::NotFaved) },
                None => Ok(SongState::NotFound)
            }
        },
        protocol::RequestAction::FAV => {
            //load song
            match possible_song {
                Some(mut song) => {
                    match db.fav_song(&mut song, &song_meta) {
                        Ok(state) => Ok(state),
                        Err(e) => Err(SoundbaseError{
                            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                            msg: format!("{:?}", e)
                        })
                    }
                },
                None => {
                    //song does not yet exist, so create it
                    let mut song = {
                        create_song_from_raw(db, &song_info)?
                    };
                    song.is_faved = true;
                    db.save(&mut song)?;
                    Ok(SongState::NowFaved)
                }
            }
        },
        protocol::RequestAction::UNFAV => {
            match possible_song {
                Some(mut song) => {
                    match db.unfav_song(&mut song, &song_meta) {
                        Ok(state) => Ok(state),
                        Err(e) => Err(SoundbaseError{
                            http_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                            msg: format!("{:?}", e)
                        })
                    }
                },
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