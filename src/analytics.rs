use chrono::{TimeZone, Utc};

#[allow(dead_code, unused_imports)]
#[path = "./analytics_protocol_generated.rs"]
#[allow(clippy::approx_constant)]
mod analytics_protocol_generated;

pub fn consume_analytics_message(buffer: &[u8]) {
    let message = analytics_protocol_generated::get_root_as_analytics_message(buffer);

    let origin = message.origin().unwrap_or("Unknown origin.");
    let dt = Utc.timestamp((message.timestamp() / 1000) as i64, ((message.timestamp() % 1000) * 1000000) as u32);
    let msg_type = message.payload_type();

    println!("Received a analytics message from {:?} at time {:?} of type {:?}", origin, dt.to_rfc3339(), msg_type);

    match msg_type {
        analytics_protocol_generated::AnalyticsMessageType::PageChange
        => store_page_change(message.payload_as_page_change().expect("Type was PageChange but payload wasn't!")),

        analytics_protocol_generated::AnalyticsMessageType::PlaybackChange
        => store_playback_change(message.payload_as_playback_change().expect("Type was PlaybackChange bzt payload wasn't!")),

        analytics_protocol_generated::AnalyticsMessageType::PlaybackSongChange
        => store_playback_song_change(message.payload_as_playback_song_change().expect("Type was PlaybakcSongchange but payload wasn't!")),

        _ => println!("\tReceived unknown Message Type!")
    };

    println!();
}

fn store_page_change(p: analytics_protocol_generated::PageChange) {
    println!("\tPage change from {:?} to {:?}", p.origin(), p.destination());
}

fn store_playback_change(p: analytics_protocol_generated::PlaybackChange) {
    println!("\tPlayback change: Source: {:?}; Name: {}; Started: {}", p.source(), p.name().unwrap_or("Unknown source."), p.started());
}

fn store_playback_song_change(p: analytics_protocol_generated::PlaybackSongChange) {
    println!("\tPlayback song change: Raw: {}; Title: {}, Artist: {}, Album: {}",
             p.raw_meta().unwrap_or("No raw meta."),
             p.title().unwrap_or("No title given."),
             p.artist().unwrap_or("No artist given."),
             p.album().unwrap_or("No Album given.")
    )
}