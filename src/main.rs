extern crate discogs;
extern crate regex;
use discogs::*;
use std::env;

pub const USER_AGENT: &'static str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn parse_release_id(arg: &str) -> Option<u32> {
    let mut splits = arg.split('/').collect::<Vec<_>>();
    let release_id_parse = splits.pop().unwrap().parse::<u32>();
    if let Ok(release_id) = release_id_parse {
        if splits.is_empty() || splits.pop() == Some("release") {
            return Some(release_id);
        }
    }
    None
}

fn trim_artist(artist_str: &str) -> String {
    let re = regex::Regex::new(r"(?P<artist>.*?)(\s\(\d*\))*$").unwrap();
    let mat = re.captures(artist_str).unwrap();
    mat["artist"].to_string()
}

fn parse_release_info(release: discogs::data_structures::Release) {
    println!("YEAR: {}", release.year);
    println!("GENRE: {}", release.genres.unwrap().pop().unwrap());
    println!(
        "ARTIST: {}",
        trim_artist(&release.artists.unwrap().pop().unwrap().name)
    );
    println!("ALBUM: {}", release.title);
    println!("COUNTRY: {}", release.country.unwrap());
    println!("TRACKLIST: {:?}", release.tracklist);
}

fn release_info(client: &mut Discogs, release_id: u32) {
    let release_result = client.release(release_id).get();
    match release_result {
        Ok(release) => {
            parse_release_info(release);
        }
        Err(_) => {
            println!("Release not found");
        }
    }
}

fn main() {
    let mut release_ids = env::args().map(|x| parse_release_id(&x)).skip(1);
    println!("{:?}", release_ids.next());
    println!("{:?}", release_ids.next());

    let mut client = Discogs::new(USER_AGENT);
    release_info(&mut client, 8492202);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_release_id_test() {
        assert_eq!(parse_release_id(&mut "".to_string()), None);
        assert_eq!(parse_release_id(&mut "abcde".to_string()), None);
        assert_eq!(parse_release_id(&mut "-1".to_string()), None);
        assert_eq!(parse_release_id(&mut "0".to_string()), Some(0));
        assert_eq!(parse_release_id(&mut "100".to_string()), Some(100));
        assert_eq!(parse_release_id(&mut "abcde/1".to_string()), None);
        assert_eq!(
            parse_release_id(
                &mut "https://www.discogs.com/My-Bloody-Valentine-Loveless/release/243919"
                    .to_string()
            ),
            Some(243919)
        );
    }

    #[test]
    fn trim_artist_test() {
        assert_eq!(trim_artist("Artist"), "Artist");
        assert_eq!(trim_artist("Artist (2)"), "Artist");
        assert_eq!(trim_artist("Artist (100)"), "Artist");
    }

    #[test]
    fn parse_release_info_test() {
        use discogs::data_structures::*;
        let mut release = Release::new(
            1234,
            "title".to_string(),
            "release".to_string(),
            "released_formatted".to_string(),
            "release_resource_url".to_string(),
            "date_added".to_string(),
            "date_changed".to_string(),
            "url".to_string(),
            2000,
            vec![Artist::new(
                123,
                "artist_name".to_string(),
                "artist_resource_url".to_string(),
            )],
            Status::Accepted,
        );
        release.genres = Some(vec!["genre".to_string()]);
        release.country = Some("Japan".to_string());
        release.tracklist = Some(vec![
            Track {
                duration: "1:00".to_string(),
                position: "1".to_string(),
                title: "title1".to_string(),
                type_: "type".to_string(),
                extra_artists: None,
            },
            Track {
                duration: "3:00".to_string(),
                position: "2".to_string(),
                title: "title2".to_string(),
                type_: "type".to_string(),
                extra_artists: None,
            },
        ]);
        parse_release_info(release);
    }

}
