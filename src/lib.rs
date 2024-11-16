use regex::Regex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use scraper::{Html, Selector};

#[cfg(test)] mod tests;    // tells cargo to not compile the tests module with cargo build but only when executing cargo test

#[allow(dead_code)]
pub struct GeniusApi {
    client_id: String,
    client_secret: String,
    access_token: String,
    http_client: reqwest::blocking::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub meta: MetaResponse,
    pub response: Response,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaResponse {
    pub status: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub hits: Vec<Highlight>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Highlight {
    pub result: HighlightResult
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HighlightResult {
    pub title: String,
    pub full_title: String,
    pub artist_names: String,
    pub id: u32,
    pub api_path: String,
    pub lyrics_state: String,
    pub song_art_image_url: String,
    #[serde(alias = "url")] pub lyrics_url: String,        // URL of the lyrics page to scrape with this code: document.querySelectorAll("div[data-lyrics-container=true]")
}

impl GeniusApi {
    pub fn new() -> Self {
        // TODO implement some sort of token caching system
        let client_id = read_from_env_file("GENIUS_CLIENT_ID");
        let client_secret = read_from_env_file("GENIUS_CLIENT_SECRET");
        let access_token = read_from_env_file("GENIUS_ACCESS_TOKEN");

        let http_client = reqwest::blocking::Client::new();

        Self {
            client_id,
            client_secret,
            access_token,
            http_client,
        }
    }

    pub fn search_songs(&self, search_term: &str) -> Result<ApiResponse, String> {
        let url = format!("https://api.genius.com/search?q={search_term}");

        match self.search_data_type::<ApiResponse>(&url) {
            Ok(r) => Ok(r),
            Err(err) => Err(err),
        }
    }

    pub fn search_song_first_res(&self, search_term: &str) -> Result<HighlightResult, String> {
        let songs = match self.search_songs(search_term) {
            Ok(s) => s,
            Err(err) => return Err(err),
        };

        match songs.response.hits.get(0) {
            Some(s) => return Ok(s.result.clone()),
            None => return Err("no songs returned from search".to_string()),
        }
    }

    pub fn scrape_song_lyrics(&self, song_url: &str) -> Option<String> {
        let song_page = self.http_client.get(song_url)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let document = Html::parse_document(&song_page);
        let selector = Selector::parse("div[data-lyrics-container=true]").unwrap();

        let lyrics = document
            .select(&selector)
            .map(|element| element.text().collect::<Vec<_>>().join("\n"))
            .collect::<Vec<_>>()
            .join("\n\n");

        //println!("{lyrics}");
        Some(lyrics)
    }

    pub fn scrape_song_lyrics_processed(&self, song_url: &str) -> Option<String> {
        let lyrics = self.scrape_song_lyrics(song_url);
        
        if let Some(lyrics) = lyrics {
            return Some(process_song_lyrics(lyrics));
        }

        return None;
    }

    fn search_data_type<T>(&self, url: &str) -> Result<T, String> where T: Serialize + DeserializeOwned + std::fmt::Debug {
        //let token_header = format!("Bearer {}", self.token);
        let url = format!("{url}&access_token={}", self.access_token);

        let res = self.http_client
            .get(url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            //.header("Authorization", token_header)
            .send();

        let Ok(res) = res else {
            return Result::Err("could not receive response".to_string());
        };

        if !res.status().is_success() {
            let err= format!("unsuccessful request, status: {}", res.status());
            return Result::Err(err);
        }

        let body = res.json::<T>();

        let Ok(data) = body else {
            return Result::Err("could not deserialize json body".to_string());
        };

        Result::Ok(data)

    }

}

fn read_from_env_file(var_name: &str) -> String {
    dotenvy::var(var_name).unwrap_or_else(|_| {
        eprintln!("could not read {var_name} from .env file");
        String::new()
    })
}

fn process_song_lyrics(lyrics: String) -> String {
    let annotations_regex = Regex::new(r"\[.*?\]")
        .expect("invalid regex pattern");

    let lyrics = annotations_regex.replace_all(&lyrics, "").to_string();

    let return_regex = Regex::new(r"\n\n")
        .expect("invalid regex pattern");

    let lyrics = return_regex.replace_all(&lyrics, r"\n").to_string();

    let return_regex = Regex::new(r"\n")
        .expect("invalid regex pattern");

    return_regex.replace_all(&lyrics, r" ")
        .trim()
        .to_string()
}


