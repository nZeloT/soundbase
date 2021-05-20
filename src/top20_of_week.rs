use crate::song_db::{TopOfTheWeekEntry, FindArtist, SongDB, FindUnique, Artist, Save, FindSong, Song};
use std::process::{Command, Stdio};
use crate::error::{Result, SoundbaseError};
use regex::Regex;
use chrono::Datelike;
use std::io::Write;

pub trait TopOfTheWeek {
    fn get_current_top_of_week() -> Vec<TopOfTheWeekEntry>;
    fn get_top_of_week(year: u16, week: u8) -> Vec<TopOfTheWeekEntry>;
}

#[derive(Debug)]
struct Top20Entry {
    position: String,
    title: String,
    artist: String,
}

pub fn fetch_new_rockantenne_top20_of_week(db: &mut SongDB<'_>) -> Result<()> {
    println!("Fetching the new top 20!");

    let top20page_body = reqwest::blocking::get("https://www.rockantenne.de/aktionen/top-20")?.text()?;
    let top20page_body_parsed = scraper::Html::parse_document(&top20page_body);

    println!("Fetched Top20 Page Body");

    let img_url = select_image_url(&top20page_body_parsed)?;
    println!("Determined image URL => {}", img_url);

    let img_data = reqwest::blocking::get(img_url)?.bytes()?;
    println!("Fetched Top20 image");

    let output = execute_tesseract(img_data.to_vec())?;

    let pattern = Regex::new("([0-9 .]+) ([A-Za-z0-9.() &']+) - \"([A-Za-z0-9.() &']+)\"")?;

    let entries = output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.trim().replace("“", "\"").replace("”", "\"").replace("|", "I"))
        .filter(|line| pattern.is_match(&line))
        .map(|line| {
            let cap = pattern.captures(&line).expect("Expected match as the mapping is after filtering out all non matches!");
            let mut pos_str = cap[1].to_string();
            let artist_str = cap[2].to_string();
            let title_str = cap[3].to_string();

            pos_str = pos_str.replace(".", "").trim().to_string();

            Top20Entry {
                position: pos_str,
                artist: artist_str,
                title: title_str,
            }
        }).collect::<Vec<Top20Entry>>();

    //determine year and week of year
    let iso_week = chrono::Utc::today().iso_week();
    let year = iso_week.year();
    let week = iso_week.week();

    for e in entries {
        println!("{:?}", e);
        store_entry_to_db(db, &e, year, week)?;
    };

    println!("Printed Output!");

    Ok(())
}

fn select_image_url(body: &scraper::Html) -> Result<String> {
    let img_selector = get_selector("main > div.row > div > figure > img")?;
    let possible_img = body.select(&img_selector).next();
    match possible_img {
        Some(img_el) => {
            match img_el.value().attr("srcset") {
                Some(s) => {
                    let values = s.split(' ').collect::<Vec<&str>>();
                    let possible_url = values.iter().find(|v| v.starts_with("http"));
                    match possible_url {
                        Some(url) => {
                            Ok(url.to_string())
                        },
                        None => Err(SoundbaseError::new("Couldn't find URL in 'srcset' image attribute!"))
                    }
                },
                None => Err(SoundbaseError::new("No attribute named 'srcset' found in image element!"))
            }
        },
        None => Err(SoundbaseError::new("No Element found with image element selector!"))
    }
}

fn execute_tesseract(stdin: Vec<u8>) -> Result<String> {
    let mut child = Command::new("tesseract")
        .arg("stdin").arg("stdout")
        .arg("-l").arg("deu+eng")
        .arg("--psm").arg("6")
        .arg("--dpi").arg("96")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let child_in = child.stdin.as_mut()
            .ok_or_else(|| SoundbaseError::new("Failed to get mutable stdin to tesseract child process!"))?;
        child_in.write_all(stdin.as_ref())?;
    }
    let output = child.wait_with_output()?;

    println!("Executed tesseract!");

    if !output.status.success() {
        return Err(SoundbaseError::new("Failed to run tesseract!"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn store_entry_to_db(db: &mut SongDB<'_>, e : &Top20Entry, year: i32, week: u32) -> Result<()> {
    //1. check whether the artist exists
    let artist_query = FindArtist::new(&e.artist);
    let db_artist = match db.find_unique(&artist_query)? {
        Some(a) => {
            println!("Found existing Artist => {:?}", a);
            a
        }
        None => {
            let mut a = Artist::new(e.artist.clone(), "".to_string());
            db.save(&mut a)?;
            a
        }
    };

    //2. check whether the song exists
    let find_song = FindSong::new(&e.title, &db_artist, None);
    let db_song = match db.find_unique(&find_song)? {
        Some(s) => {
            println!("Found existing song => {:?}", s);
            s
        }
        None => {
            let mut s = Song::new(e.title.clone(), "".to_string(), db_artist);
            db.save(&mut s)?;
            s
        }
    };

    let pos = e.position.parse::<u8>()?;

    //3. insert top20 entry
    let mut tow_entry = TopOfTheWeekEntry::new(year as u16, week as u8, "Rock Antenne".to_string(), pos, db_song);
    db.save(&mut tow_entry)?;
    println!("Successfully stored.");
    Ok(())
}

fn get_selector(selector: &'static str) -> Result<scraper::Selector> {
    let sel = scraper::Selector::parse(selector);
    match sel {
        Ok(s) => Ok(s),
        Err(e) => {
            Err(SoundbaseError{
                http_code: tide::StatusCode::InternalServerError,
                msg: format!("{:?}", e)
            })
        }
    }
}