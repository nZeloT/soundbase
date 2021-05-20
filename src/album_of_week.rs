use crate::error::{Result, SoundbaseError};
use crate::song_db::{AlbumOfTheWeek, SongDB, Query, QueryBounds, QueryOrdering, OrderDirection, Save, Artist, FindUnique, FindArtist, FindAlbum, Album};

pub trait AlbumsOfTheWeek {
    fn get_current_album_of_week(&mut self) -> Result<Option<AlbumOfTheWeek>>;
    fn get_albums_of_week(&mut self, offset: u8, limit: u8) -> Result<Vec<AlbumOfTheWeek>>;
    fn store_new_album_of_week(&mut self, album: &mut AlbumOfTheWeek) -> Result<()>;
}

impl<'a> AlbumsOfTheWeek for SongDB<'a> {
    fn get_current_album_of_week(&mut self) -> Result<Option<AlbumOfTheWeek>> {
        let mut albums = self.query(
            QueryBounds { offset: 0, page_size: 1 },
            None,
            Some(QueryOrdering{
                direction: OrderDirection::Desc,
                on_field: "source_date".to_string()
            })
        )?;
        Ok(albums.pop())
    }

    fn get_albums_of_week(&mut self, offset: u8, limit: u8) -> Result<Vec<AlbumOfTheWeek>> {
        let albums = self.query(
            QueryBounds{
                offset: offset as u64,
                page_size: limit as u16
            },
            None,
            Some(QueryOrdering{
                direction: OrderDirection::Desc,
                on_field: "source_date".to_string()
            })
        )?;
        Ok(albums)
    }

    fn store_new_album_of_week(&mut self, album: &mut AlbumOfTheWeek) -> Result<()> {
        self.save(album)?;
        Ok(())
    }
}

pub fn fetch_new_rockantenne_album_of_week(db: &mut SongDB<'_>) -> Result<()> {
    //1. fetch overview page
    let overview_body = reqwest::blocking::get("https://www.rockantenne.de/musik/album-der-woche/")?.text()?;
    let overview_body_parsed = scraper::Html::parse_document(&overview_body);

    //3. from the top post extract
    //  3.1 artist
    // at the same time remove the " - " at the end of the artist string
    let artist = select_artist(&overview_body_parsed)?.trim_end_matches(" - ").to_string();

    //  3.2 album
    let album = select_album(&overview_body_parsed)?;
    //  3.3 date
    let date_string = select_date(&overview_body_parsed)?;
    //  3.4 link to full post
    let full_post_url = select_full_post_link(&overview_body_parsed)?;

    //4. fetch full post
    let mut url = "https://www.rockantenne.de".to_string();
    url += &full_post_url;
    println!("Requesting full post from => {}", url);
    let full_body = reqwest::blocking::get(&url)?.text()?;
    let full_body_parsed = scraper::Html::parse_document(&full_body);
    //5. from full post extract
    //  5.1 comment/reasoning
    let reasoning_html = select_reasoning_html(&full_body_parsed)?;
    let reasoning_text = select_reasoning_text(&full_body_parsed)?;
    //  5.2 song list
    let song_list_html = select_song_list(&full_body_parsed)?;

    println!("New Album of the Week:");
    println!("\tArtist => {}", artist);
    println!("\tAlbum => {}", album);
    println!("\tDate => {}", date_string);
    println!("\tFull Post URI => {}", full_post_url);
    println!("\tReasoning HTML => {}", reasoning_html);
    println!("\tReasoning Text => {:?}", reasoning_text);
    println!("\tSong List => {}", song_list_html);
    println!();

    //6. check for artist, if not write new
    let artist_query = FindArtist::new(&artist);
    let db_artist = match db.find_unique(&artist_query)? {
        Some(a) => {
            println!("Found existing Artist => {:?}", a);
            a
        },
        None => {
            let mut a = Artist::new(artist, "".to_string());
            db.save(&mut a)?;
            println!("Stored new Artist => {:?}", a);
            a
        }
    };

    //7. check for album if not write new
    let album_query = FindAlbum::new(&album, &db_artist);
    let db_album = match db.find_unique(&album_query)? {
        Some(a) => {
            println!("Found existing Album => {:?}", a);
            a
        },
        None => {
            let mut a = Album::new(album, "".to_string(), db_artist)?;
            db.save(&mut a)?;
            println!("Stored new Album => {:?}", a);
            a
        }
    };

    //8. write new album of week entry
    let new_aofw_date = chrono::DateTime::parse_from_rfc3339(&date_string)?;
    let mut new_aofw = AlbumOfTheWeek::new("Rock Antenne".to_string(), reasoning_html, new_aofw_date, db_album, song_list_html);
    match db.get_current_album_of_week()? {
        Some(aofw) => {
            //to prevent double entries of the same album first check for existence
            if new_aofw == aofw {
                return Err(SoundbaseError::new("Album of Week already found in DB. Skipping."))
            }
        },
        None => {}
    };

    db.save(&mut new_aofw)?;
    println!("Stored new Album of Week => {:?}", new_aofw);
    Ok(())
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
        },
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