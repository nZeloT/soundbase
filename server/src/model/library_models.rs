
#[derive(Clone, Debug)]
pub struct SimpleTrack {
    pub track_id : i32,
    pub title : String,
    pub album : SimpleAlbum,
    pub artists : Vec<SimpleArtist>,
    pub is_faved : bool,
    pub duration_ms : i64
}

#[derive(Clone, Debug)]
pub struct SimpleAlbum {
    pub album_id : i32,
    pub name : String
}

#[derive(Clone, Debug)]
pub struct SimpleArtist {
    pub artist_id : i32,
    pub name : String
}