use crate::analytics_protocol_generated;
use crate::error;
use crate::analytics;

pub fn consume_analytics_message<DB>(db: &mut DB, buffer: Vec<u8>) -> error::Result<()>
    where DB: analytics::AnalyticsDB
{
    let msg = analytics_protocol_generated::root_as_analytics_message(buffer.as_slice())
        .expect("Expected AnalyticsMessage. Got something different.");

    let metadata = analytics::Metadata::new(&msg);
    println!("Received a analytics message from {:?} at time {:?} of type {:?}", metadata.tmstp, metadata.origin, metadata.kind);

    match metadata.kind {
        analytics::MessageKind::PageChange => {
            let payload = msg.payload_as_page_change().unwrap();
            let page = analytics::PageChange::new(&payload);
            println!("\tPage change from {:?} to {:?}", page.src, page.dst);
            db.store_page_change(&metadata, &page)?;
        }

        analytics::MessageKind::PlaybackChange => {
            let payload = msg.payload_as_playback_change().unwrap();
            let playback = analytics::PlaybackChange::new(&payload);
            println!("\tPlayback change: Source: {:?}; Name: {}; Started: {}", playback.source, playback.name, playback.started);
            db.store_playback_change(&metadata, &playback)?;
        }

        analytics::MessageKind::SongChange => {
            let payload = msg.payload_as_playback_song_change().unwrap();
            let song = analytics::SongChange::new(&payload);
            println!("\tPlayback song change: Raw: {}; Title: {}, Artist: {}, Album: {}",
                     song.raw_meta,
                     song.title,
                     song.artist,
                     song.album
            );
            db.store_song_change(&metadata, &song)?;
        }
    };

    println!();
    Ok(())
}