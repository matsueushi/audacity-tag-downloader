extern crate discogs;
extern crate dotenv;
extern crate quick_xml;
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
    use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
    use quick_xml::Writer;
    use std::io::Cursor;

    fn tag_elem<'a>(name: &str, value: String) -> Event<'a> {
        let mut elem = BytesStart::borrowed_name(b"tag");
        elem.push_attribute(("name", name));
        elem.push_attribute(("value", value.as_str()));
        Event::Start(elem)
    }

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let decl = BytesDecl::new(b"1.0", None, None);
    writer.write_event(Event::Decl(decl)).is_ok();
    writer
        .write_event(Event::Start(BytesStart::borrowed_name(b"tags")))
        .is_ok();
    writer
        .write_event(tag_elem("YEAR", release.year.to_string()))
        .is_ok();
    if let Some(mut genres) = release.genres {
        if let Some(primary_genre) = genres.pop() {
            writer.write_event(tag_elem("GENRE", primary_genre)).is_ok();
        }
    }
    if let Some(mut artists) = release.artists {
        if let Some(primary_artist) = artists.pop() {
            writer
                .write_event(tag_elem("ARTIST", trim_artist(&primary_artist.name)))
                .is_ok();
        }
    }
    writer.write_event(tag_elem("ALBUM", release.title)).is_ok();
    if let Some(country) = release.country {
        writer.write_event(tag_elem("COUNTRY", country)).is_ok();
    }
    if let Some(mut images) = release.images {
        if let Some(primary_image) = images.pop() {
            println!("IMAGE: {}", primary_image.resource_url);
        }
    }
    writer
        .write_event(Event::End(BytesEnd::borrowed(b"tags")))
        .is_ok();
    if let Some(tracks) = release.tracklist {
        for t in tracks {
            println!("{},{},{}", t.duration, t.position, t.title)
        }
    }

    println!("{:?}", String::from_utf8(writer.into_inner().into_inner()));
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
    dotenv::dotenv().ok();
    let _ = env::args().map(|x| parse_release_id(&x)).skip(1);

    let mut client = Discogs::new(USER_AGENT);
    if let Ok(consumer_key) = env::var("CONSUMER_KEY") {
        client.key(&consumer_key);
    }
    if let Ok(consumer_secret) = env::var("CONSUMER_SECRET") {
        client.secret(&consumer_secret);
    }
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
        assert_eq!(trim_artist("3776"), "3776");
        assert_eq!(trim_artist("3776 (2)"), "3776")
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
        release.images = Some(vec![Image {
            resource_url: "url".to_string(),
            image_type: "primary".to_string(),
            uri: "uri".to_string(),
            uri150: "uri150".to_string(),
            height: 80,
            width: 100,
        }]);
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
