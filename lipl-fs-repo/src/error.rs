use futures::channel::oneshot::Canceled;
use thiserror::{Error};

#[derive(Error, Debug)]
pub enum FileRepoError {
    #[error("File {0:?} has invalid filestem")]
    Filestem(Option<String>),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Lyric with id {1} not found. Cannot add to playlist with id {0}")]
    PlaylistInvalidMember(String, String),

    #[error("Cannot find directory {0:?}")]
    CannotFindDirectory(Option<String>),

    #[error("Send failed for {0}")]
    SendFailed(String),

    #[error("Canceled")]
    Canceled(#[from] Canceled),

    #[error("Parse error for {0}")]
    Parse(String),
}