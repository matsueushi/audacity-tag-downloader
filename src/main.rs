extern crate discogs;
use discogs::*;
use std::env;

pub const USER_AGENT: &'static str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() {
    println!("{}", USER_AGENT);

    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        println!("{:?}", args[i].split('/').collect::<Vec<_>>());
    }

    let mut client = Discogs::new(USER_AGENT);
    let release = client.release(8492202).get();

    if release.is_ok() {
        let release_result = release.ok().unwrap();
        println!("YEAR: {}", release_result.year);
        println!("GENRE: {:?}", release_result.genres);
        println!("ARTIST: {:?}", release_result.artists);
        println!("ALBUM: {}", release_result.title);
    }
}
