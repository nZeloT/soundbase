use crate::song_like;
use crate::song_like_protocol_generated;
use crate::error;
use crate::error::SoundbaseError;
use crate::db::SongDB;

pub fn consume_like_message<DB>(db: &mut DB, dissects: &[song_like::SourceMetadataDissect], buffer: Vec<u8>) -> error::Result<Vec<u8>>
    where DB: SongDB
{
    let msg = song_like_protocol_generated::root_as_song_message(buffer.as_slice())
        .expect("Expected SongMessage. Got something else!");

    print!("Received Song Like Message with id {:?} ", msg.id());

    assert_eq!(msg.payload_type(), song_like_protocol_generated::MessagePayload::Request);

    let resp_kind = process_message(db, dissects, &msg.payload_as_request().unwrap())?;

    //now build response
    build_response_message(msg.id(), resp_kind)
}

fn build_response_message(msg_id: u64, response: song_like_protocol_generated::ResponseKind) -> error::Result<Vec<u8>> {
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

fn process_message<DB>(db: &mut DB, dissects: &[song_like::SourceMetadataDissect], request: &song_like_protocol_generated::Request) -> error::Result<song_like_protocol_generated::ResponseKind>
    where DB: SongDB
{
    println!("from requesting party {:?}", request.requesting_party());

    let mut song_info = song_like::Song::new(&request.song_info());
    let song_source_info = request.song_source_info();
    let song_source = song_like::SongSource::new(&song_source_info, dissects);
    let song_meta = song_like::SongMetadata {
        origin: request.requesting_party().to_string(),
        source: song_source,
    };

    println!("\tWith given song source: {} and details: {}", &song_meta.source, song_info);
    println!("\tRequesting action: {:?}", request.action_kind());

    if song_info.has_only_raw() {
        song_info.dissect_raw_using_source_info(&song_meta.source)?;
        println!("\tDissected Metadata to => {:?}", song_info);
    }

    let resp_kind = match request.action_kind() {
        song_like_protocol_generated::RequestAction::INFO => db.get_state(&song_info),
        song_like_protocol_generated::RequestAction::FAV => db.fav_song(&song_info, &song_meta),
        song_like_protocol_generated::RequestAction::UNFAV => db.unfav_song(&song_info, &song_meta),
        _ => Err(SoundbaseError { http_code: tide::StatusCode::InternalServerError, msg: "Unknown RequestAction received!".to_string() })
    };
    match resp_kind {
        Ok(kind) => {
            println!("\tReturning song state => {:?}", kind);
            Ok(kind.into())
        },
        Err(e) => Err(e)
    }
}