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

use chrono::Datelike;
use regex::Regex;
use crate::db_new::album::AlbumDb;
use crate::db_new::album_of_week::AlbumOfWeekDb;
use crate::db_new::DbApi;
use crate::db_new::models::{NewAlbum, NewAlbumOfWeek, NewArtist};
use crate::error::{Result, SoundbaseError};

use super::get_selector;

pub async fn fetch_new_rockantenne_album_of_week(api: DbApi) -> Result<()>
{
    //1. fetch overview page
    let overview_body = reqwest::get("https://www.rockantenne.de/musik/album-der-woche/").await?.text().await?;
    let (artist, album_name, date_string, full_post_url) = parse_overview_body(&*overview_body)?;

    //4. fetch full post
    let mut url = "https://www.rockantenne.de".to_string();
    url += &full_post_url;
    println!("Requesting full post from => {}", url);
    let full_body = reqwest::get(&url).await?.text().await?;

    let (reasoning_html, song_list_html) = parse_full_post_body(&*full_body)?;

    let album_track_count = get_track_count_from_song_list(&*song_list_html)?;

    println!("New Album of the Week:");
    println!("\tArtist => {}", artist);
    println!("\tAlbum => {}", album_name);
    println!("\tDate => {}", date_string);
    println!("\tTrack Count => {}", album_track_count);
    println!("\tFull Post URI => {}", full_post_url);
    println!("\tReasoning HTML => {}", reasoning_html);
    println!("\tSong List => {}", song_list_html);
    println!();

    let new_aofw_date = chrono::DateTime::parse_from_rfc3339(&date_string)?;
    if has_db_entry_for_week(&api, new_aofw_date.year(), new_aofw_date.iso_week().week() as i32)? {
        let artist = api.get_or_create_artist(&*artist, || NewArtist { name: &*artist, spot_id: None })?;
        let album = api.get_or_create_album(&artist, &*album_name, || {
            NewAlbum {
                name: &*album_name,
                year: new_aofw_date.year(),
                spot_id: None,
                was_aow: None,
                album_type: None,
                is_faved: Some(false),
                total_tracks: album_track_count,
            }
        })?;

        let _ = api.set_was_aow(&album, true)?;
        
        let _ = api.new_album_of_week(NewAlbumOfWeek{
            album_id: album.album_id,
            year: new_aofw_date.year(),
            week: new_aofw_date.iso_week().week() as i32,
            source_name: "Rock Antenne",
            source_date: &*new_aofw_date.to_rfc3339(),
            source_comment: &*reasoning_html,
            track_list_raw: Some(song_list_html)
        })?;

    } else {
        println!("This week already has an AOW for Rock Antenne; Skipping this one.");
    }

    Ok(())
}

fn parse_overview_body(html: &str) -> Result<(String, String, String, String)> {
    let overview_body_parsed = scraper::Html::parse_document(html);

    //3. from the top post extract
    //  3.1 artist
    // at the same time remove the " - " at the end of the artist string
    let artist = select_artist(&overview_body_parsed)?.trim().trim_end_matches("-").trim_end_matches("–").trim().to_string();

    //  3.2 album
    let album = select_album(&overview_body_parsed)?;
    //  3.3 date
    let date_string = select_date(&overview_body_parsed)?;
    //  3.4 link to full post
    let full_post_url = select_full_post_link(&overview_body_parsed)?;

    Ok((artist, album, date_string, full_post_url))
}

fn parse_full_post_body(html: &str) -> Result<(String, String)> {
    let full_body_parsed = scraper::Html::parse_document(html);
    //5. from full post extract
    //  5.1 comment/reasoning
    let reasoning_html = select_reasoning_html(&full_body_parsed)?;
    //  5.2 song list
    let song_list_html = select_song_list(&full_body_parsed)?;

    Ok((reasoning_html, song_list_html))
}

fn get_track_count_from_song_list(song_list_html: &str) -> Result<i32> {
    let plain_songs = song_list_html
        .replace("<p>", "")
        .replace("</p>", "")
        .replace("<br>", "")
        .replace("\t", "");
    let songs = plain_songs.split("\n").collect::<Vec<&str>>();
    let pattern = Regex::new("([0-9]+)\\. .+")?;
    match songs.last() {
        Some(&last) => {
            match pattern.captures(last) {
                Some(cap) => {
                    let num_str = &cap[1];
                    let num = num_str.parse::<i32>()?;
                    Ok(num)
                }
                None => Err(SoundbaseError::new("Couldn't match regex for track count on AOW!"))
            }
        }
        None => Err(SoundbaseError::new("Didn't find the last song of album!"))
    }
}

fn has_db_entry_for_week(db : &DbApi, year : i32, iso_week : i32) -> Result<bool> {
    let api : &dyn AlbumOfWeekDb = db;
    let result = api.find_by_source_and_week("Rock Antenne", year, iso_week)?;
    match result {
        Some(_) => Ok(true),
        None => Ok(false)
    }
}

fn select_artist(overview: &scraper::Html) -> Result<String> {
    let artist_selector = get_selector("article.teaser_item.width-12 > div.teaser_item__content > h3 > a > span")?;
    let possible_artist = overview.select(&artist_selector).next();
    match possible_artist {
        Some(artist_el) => {
            match artist_el.text().next() {
                Some(text) => Ok(text.to_string()),
                None => Err(SoundbaseError::new("No Text found in artist element selector!"))
            }
        }
        None => Err(SoundbaseError::new("No Element found in artist element selector!"))
    }
}

fn select_album(overview: &scraper::Html) -> Result<String> {
    let album_selector = get_selector("article.teaser_item.width-12 > div.teaser_item__content > h3 > a > span > em")?;
    let possible_album = overview.select(&album_selector).next();
    match possible_album {
        Some(album_el) => Ok(album_el.inner_html()),
        None => Err(SoundbaseError::new("No Element found in album element selector!"))
    }
}

fn select_date(overview: &scraper::Html) -> Result<String> {
    let date_selector = get_selector("article.teaser_item.width-12 > div.teaser_item__content > h3 > small.teaser_item__date")?;
    let possible_date = overview.select(&date_selector).next();
    match possible_date {
        Some(date_el) => {
            match date_el.value().attr("content") {
                Some(c) => Ok(c.to_string()),
                None => Err(SoundbaseError::new("No Attribute named content found in date element!"))
            }
        }
        None => Err(SoundbaseError::new("No Element found in date element selector!"))
    }
}

fn select_full_post_link(overview: &scraper::Html) -> Result<String> {
    let link_selector = get_selector("article.teaser_item.width-12 > div.teaser_item__content > h3 > a")?;
    let possible_link = overview.select(&link_selector).next();
    match possible_link {
        Some(link_el) => {
            match link_el.value().attr("href") {
                Some(c) => Ok(c.to_string()),
                None => Err(SoundbaseError::new("No Attribute named href found in full post link element!"))
            }
        }
        None => Err(SoundbaseError::new("No Element found in full post link element selector!"))
    }
}

fn select_reasoning_html(full_post: &scraper::Html) -> Result<String> {
    let reasoning_selector = get_selector("div.pagecontent > div.row > main > div > div.row > div.small-12.columns > div.clearfix + p + div + div.clearfix")?;
    let possible_reasoning = full_post.select(&reasoning_selector).next();
    match possible_reasoning {
        Some(reasoning_el) => Ok(reasoning_el.inner_html()),
        None => Err(SoundbaseError::new("No Element found in reasoning element selector!"))
    }
}

fn select_reasoning_text(full_post: &scraper::Html) -> Result<Vec<&str>> {
    let reasoning_selector = get_selector("div.pagecontent > div.row > main > div > div.row > div.small-12.columns > div.clearfix + p + div + div.clearfix")?;
    let possible_reasoning = full_post.select(&reasoning_selector).next();
    match possible_reasoning {
        Some(reasoning_el) => Ok(reasoning_el.text().collect::<Vec<_>>()),
        None => Err(SoundbaseError::new("No Element found in reasoning element selector!"))
    }
}

fn select_song_list(full_post: &scraper::Html) -> Result<String> {
    let song_list_selector = get_selector("div.pagecontent > div.row > main > div > div.row > div.small-12 > div.clearfix:nth-child(2)")?;
    let possible_song_list = full_post.select(&song_list_selector).next();
    match possible_song_list {
        Some(song_list_el) => Ok(song_list_el.inner_html()),
        None => Err(SoundbaseError::new("No Element found in song list element selector!"))
    }
}