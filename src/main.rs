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

#[derive(Clone, Debug)]
struct SongData {
    title: String,
    xml: Vec<u8>,
}

#[derive(Clone, Debug)]
struct AlbumData {
    song_data_vec: Vec<SongData>,
    image_url: Option<String>,
}

fn parse_release_data(release: discogs::data_structures::Release) -> AlbumData {
    use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
    use quick_xml::Writer;
    use std::io::Cursor;

    fn write_tag_elem<'a>(writer: &mut Writer<Cursor<Vec<u8>>>, name: &str, value: String) {
        let mut elem = BytesStart::borrowed_name(b"tag");
        elem.push_attribute(("name", name));
        elem.push_attribute(("value", value.as_str()));
        writer.write_event(Event::Empty(elem)).ok();
    }

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 4);
    let decl = BytesDecl::new(b"1.0", None, None);
    writer.write_event(Event::Decl(decl)).is_ok();
    writer
        .write_event(Event::Start(BytesStart::borrowed_name(b"tags")))
        .is_ok();

    write_tag_elem(&mut writer, "YEAR", release.year.to_string());
    if let Some(mut genres) = release.genres {
        if let Some(primary_genre) = genres.pop() {
            write_tag_elem(&mut writer, "GENRE", primary_genre);
        }
    }
    if let Some(mut artists) = release.artists {
        if let Some(primary_artist) = artists.pop() {
            write_tag_elem(&mut writer, "ARTIST", trim_artist(&primary_artist.name));
        }
    }
    write_tag_elem(&mut writer, "ALBUM", release.title);
    if let Some(country) = release.country {
        write_tag_elem(&mut writer, "COUNTRY", country);
    }

    let mut song_data_vec = Vec::new();
    if let Some(tracks) = release.tracklist {
        for t in tracks {
            let mut writer_t = writer.clone();
            write_tag_elem(&mut writer_t, "TITLE", t.title.clone());
            write_tag_elem(&mut writer_t, "TRACKNUMBER", t.position);
            writer_t
                .write_event(Event::End(BytesEnd::borrowed(b"tags")))
                .is_ok();
            song_data_vec.push(SongData {
                title: t.title,
                xml: writer_t.into_inner().into_inner(),
            });
        }
    }

    let mut image_url: Option<String> = None;
    if let Some(mut images) = release.images {
        if let Some(primary_image) = images.pop() {
            image_url = Some(primary_image.resource_url);
        }
    }
    AlbumData {
        song_data_vec: song_data_vec,
        image_url: image_url,
    }
}

fn write_release_data(client: &mut Discogs, release_id: u32) {
    use std::io::Write;
    let release_result = client.release(release_id).get();
    match release_result {
        Ok(release) => {
            let release_data = parse_release_data(release);

            for (i, song_data) in release_data.song_data_vec.iter().enumerate() {
                if let Ok(mut file) =
                    std::fs::File::create(format!("{:02} {}.xml", i, song_data.title))
                {
                    file.write_all(&song_data.xml);
                }
            }
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
    write_release_data(&mut client, 8492202);
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
            resource_url: "image_url".to_string(),
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
        let release_info = parse_release_data(release);
        assert_eq!(release_info.image_url, Some("image_url".to_string()));
    }

}
