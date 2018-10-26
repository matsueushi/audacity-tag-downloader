extern crate discogs;
use discogs::*;
use std::env;

pub const USER_AGENT: &'static str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn release_info(client: &mut Discogs, release_id: u32) {
    let release = client.release(release_id).get();

    if release.is_ok() {
        let release_result = release.ok().unwrap();
        println!("YEAR: {}", release_result.year);
        println!("GENRE: {:?}", release_result.genres);
        println!("ARTIST: {:?}", release_result.artists);
        println!("ALBUM: {}", release_result.title);
        println!("COUNTRY: {:?}", release_result.country);
    }
}

fn parse_release_id(arg: &mut String) -> Option<u64> {
    let mut splits = arg.split('/').collect::<Vec<_>>();
    let release_id_parse = splits.pop().unwrap().parse::<u64>();
    if let Ok(release_id) = release_id_parse {
        if splits.is_empty() || splits.pop() == Some("release") {
            return Some(release_id);
        }
    }
    None
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    for i in 0..args.len() {
        parse_release_id(&mut args[i]);
    }

    let mut client = Discogs::new(USER_AGENT);
    release_info(&mut client, 8492202);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_release_id_test() {
        assert_eq!(parse_release_id(&mut "-1".to_string()), None);
        assert_eq!(parse_release_id(&mut "0".to_string()), Some(0));
        assert_eq!(parse_release_id(&mut "100".to_string()), Some(100));
        assert_eq!(parse_release_id(&mut "y/1".to_string()), None);
        assert_eq!(
            parse_release_id(
                &mut "https://www.discogs.com/My-Bloody-Valentine-Loveless/release/243919"
                    .to_string()
            ),
            Some(243919)
        );
    }

}
