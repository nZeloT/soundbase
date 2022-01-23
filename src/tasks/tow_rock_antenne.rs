/*
 * Copyright 2022 nzelot<leontsteiner@gmail.com>
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

use std::io::Write;
use std::process::{Command, Stdio};

use chrono::Datelike;
use regex::Regex;

use crate::db_new::DbApi;
use crate::db_new::models::NewArtist;
use crate::error::Error;
use crate::string_utils::{UnifyApostrophes, UnifyQuotes};
use crate::Result;

use super::get_selector;

#[derive(Debug)]
struct Top20Entry {
    position: String,
    title: String,
    artist: String,
}

pub async fn fetch_new_rockantenne_top20_of_week(db: DbApi) -> Result<()>
{
    println!("Fetching the new top 20!");

    let top20page_body = reqwest::get("https://www.rockantenne.de/aktionen/top-20").await?.text().await?;
    println!("Fetched Top20 Page Body");

    let img_url = select_image_url(&*top20page_body)?;
    println!("Determined image URL => {}", img_url);

    let img_data = reqwest::get(img_url).await?.bytes().await?;
    println!("Fetched Top20 image");

    //determine year and week of year
    let iso_week = chrono::Utc::today().iso_week();
    let year = iso_week.year();
    let week = iso_week.week();

    // if db.has_charts_for_week("Rock Antenne", year, week as i32)? {
    //     println!("Already found charts for this week; Skipping");
    //     return Ok(());
    // }

    let output = execute_tesseract(img_data.to_vec())?;

    let pattern = Regex::new("([0-9 .]+) (.+) - \"(.+)\"")?;

    output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.trim().unify_quotes().unify_apostrophes().replace("|", "I"))
        .for_each(|line| {
            match pattern.captures(&line) {
                Some(cap) => {
                    let mut pos_str = cap[1].to_string();
                    let artist_str = cap[2].to_string();
                    let title_str = cap[3].to_string();

                    pos_str = pos_str.replace(".", "").trim().to_string();
                    if pos_str == "111" {
                        pos_str = "11".to_string();
                    }

                    let entry = Top20Entry {
                        position: pos_str,
                        artist: artist_str,
                        title: title_str,
                    };
                    if let Err(err) = store_entry_to_db(&db, &entry, year, week) {
                        println!("Received error during entry storage! => {:?}", err);
                    } else {
                        println!("Stored {:?} in DB", entry);
                    }
                }
                None => {
                    println!("Unmatched line '{}'", line);
                }
            }
        });

    println!("Printed Output!");

    Ok(())
}

fn select_image_url(html: &str) -> Result<String> {
    let body = scraper::Html::parse_document(&html);
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
                        }
                        None => Err(Error::InternalError("Couldn't find URL in 'srcset' image attribute!".to_string()))
                    }
                }
                None => Err(Error::InternalError("No attribute named 'srcset' found in image element!".to_string()))
            }
        }
        None => Err(Error::InternalError("No Element found with image element selector!".to_string()))
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
            .ok_or_else(|| Error::InternalError("Failed to get mutable stdin to tesseract child process!".to_string()))?;
        child_in.write_all(stdin.as_ref())?;
    }
    let output = child.wait_with_output()?;

    println!("Executed tesseract!");

    if !output.status.success() {
        return Err(Error::InternalError("Failed to run tesseract!".to_string()));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn store_entry_to_db(db: &DbApi, e: &Top20Entry, year: i32, week: u32) -> Result<()>
{
    // let artist = db.get_or_create_artist(&*e.artist, || NewArtist {
    //     name: &*e.artist,
    //     spot_id: None,
    // })?;

    //TODO find album name by calling to spotify

    // //2. check whether the song exists
    // let db_song = match db.find_unique(FindSong::new(e.title.clone(), &db_artist, None))? {
    //     Some(s) => {
    //         println!("Found existing song => {:?}", s);
    //         s
    //     }
    //     None => {
    //         let mut s = Song::new(e.title.clone(), "".to_string(), db_artist);
    //         db.save(&mut s)?;
    //         s
    //     }
    // };
    //
    // let pos = e.position.parse::<u8>()?;
    //
    // //3. insert top20 entry
    // let mut tow_entry = TopOfTheWeekEntry::new(year as u16, week as u8, "Rock Antenne".to_string(), pos, db_song);
    // db.save(&mut tow_entry)?;
    // println!("Successfully stored.");
    Ok(())
}