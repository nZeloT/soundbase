
#[derive(Debug)]
pub struct SourceSelectedTopSong {
    title: String,
    artist: String,
    source: String,
    source_date: chrono::Date<chrono::Utc>
}

pub trait TopOfTheWeek {
    fn get_current_top_of_week() -> Vec<SourceSelectedTopSong>;
}