use tokio::fs::{read_dir, File};
use futures::stream::{Stream, StreamExt};
use futures::future::ready;
use tokio::io::BufReader;
use std::path::PathBuf;
use std::io::Error;

mod parts;
pub use parts::to_parts_async;

pub struct Lyric {
    pub id: String,
    pub yaml: Option<String>,
    pub parts: Vec<Vec<String>>,
}

pub async fn get_file(pb: &PathBuf) -> Result<Lyric, Error> {
    let file = File::open(pb).await?;
    let reader = BufReader::new(file);
    let (yaml, parts) = to_parts_async(reader).await?;
    Ok(
        Lyric {
            id: pb.file_stem().unwrap().to_string_lossy().to_string(),
            yaml,
            parts,
        }
    )
}

pub async fn get_lyrics(path: &str) -> Result<impl Stream<Item=Lyric>, Error> {
    read_dir(path)
    .await
    .map(|rd|
        rd
        .filter(|entry| ready(entry.is_ok()))
        .map(|entry| entry.unwrap().path())
        .then(|path_buffer| async move {
            get_file(&path_buffer).await
        })
        .filter(|lyric_file| ready(lyric_file.is_ok()))
        .map(|lyric_file| lyric_file.unwrap())
    )
}
