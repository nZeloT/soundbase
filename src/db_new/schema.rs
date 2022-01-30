table! {
    album_artists (id) {
        id -> Int4,
        album_id -> Int4,
        artist_id -> Int4,
    }
}

table! {
    albums (album_id) {
        album_id -> Int4,
        name -> Varchar,
        album_type -> Int4,
        year -> Int4,
        total_tracks -> Nullable<Int4>,
        is_faved -> Bool,
        is_known_spot -> Bool,
        is_known_local -> Bool,
        was_aow -> Bool,
        spot_id -> Nullable<Varchar>,
    }
}

table! {
    albums_of_week (aow_id) {
        aow_id -> Int4,
        album_id -> Int4,
        year -> Int4,
        week -> Int4,
        source_name -> Varchar,
        source_comment -> Text,
        source_date -> Varchar,
        track_list_raw -> Nullable<Text>,
    }
}

table! {
    artist_genre (id) {
        id -> Int4,
        artist_id -> Int4,
        genre_id -> Int4,
    }
}

table! {
    artists (artist_id) {
        artist_id -> Int4,
        name -> Varchar,
        is_faved -> Bool,
        is_known_spot -> Bool,
        is_known_local -> Bool,
        spot_id -> Nullable<Varchar>,
    }
}

table! {
    charts_of_week (chart_id) {
        chart_id -> Int4,
        year -> Int4,
        calendar_week -> Int4,
        source_name -> Varchar,
        track_id -> Int4,
        chart_position -> Int4,
    }
}

table! {
    genre (genre_id) {
        genre_id -> Int4,
        name -> Varchar,
    }
}

table! {
    track_artist (id) {
        id -> Int4,
        track_id -> Int4,
        artist_id -> Int4,
    }
}

table! {
    track_fav_proposals (track_fav_id) {
        track_fav_id -> Int4,
        source_name -> Varchar,
        source_prop -> Varchar,
        ext_track_title -> Varchar,
        ext_artist_name -> Varchar,
        ext_album_name -> Nullable<Varchar>,
        track_id -> Nullable<Int4>,
    }
}

table! {
    tracks (track_id) {
        track_id -> Int4,
        title -> Varchar,
        album_id -> Int4,
        disc_number -> Nullable<Int4>,
        track_number -> Nullable<Int4>,
        duration_ms -> Int8,
        is_faved -> Bool,
        local_file -> Nullable<Varchar>,
        spot_id -> Nullable<Varchar>,
    }
}

joinable!(album_artists -> albums (album_id));
joinable!(album_artists -> artists (artist_id));
joinable!(albums_of_week -> albums (album_id));
joinable!(artist_genre -> artists (artist_id));
joinable!(artist_genre -> genre (genre_id));
joinable!(charts_of_week -> tracks (track_id));
joinable!(track_artist -> artists (artist_id));
joinable!(track_artist -> tracks (track_id));
joinable!(track_fav_proposals -> tracks (track_id));
joinable!(tracks -> albums (album_id));

allow_tables_to_appear_in_same_query!(
    album_artists,
    albums,
    albums_of_week,
    artist_genre,
    artists,
    charts_of_week,
    genre,
    track_artist,
    track_fav_proposals,
    tracks,
);
