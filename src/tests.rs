/* run with "cargo test -- --nocapture" to see printed messages on stout and stderr */

use crate::GeniusApi;

#[test]
fn genius_api_struct_env_file_read() {
    let ga = GeniusApi::new();
    assert!(ga.access_token != "");
    assert!(ga.client_id != "");
    assert!(ga.client_secret != "");
}

#[test]
fn genius_api_search_song_request() {
    let ga = GeniusApi::new();
    let res = ga.search_song_first_res("Unpeeled");

    let song = match res {
        Ok(r) => {
            println!("{r:?}");
            assert!(true);
            r
        },
        Err(err)=> {
            eprintln!("{err}");
            assert!(false);
            return;
        }
    };

    let lyrics = ga.scrape_song_lyrics_processed(&song.lyrics_url);
    if let Some(lyrics) = lyrics {
        println!("{lyrics}");
        assert!(true);
    }
    else {
        assert!(false);
    }
}
