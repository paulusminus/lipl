use std::fmt::Debug;
use std::path::PathBuf;

use async_trait::async_trait;

pub use error::FileRepoError;
use fs::IO;
use futures::{channel::mpsc};
use futures::{FutureExt, StreamExt, TryStreamExt, TryFutureExt, Future};
use lipl_types::{
    time_it, LiplRepo, Lyric, Playlist, error::{ModelError, ModelResult}, Summary, Uuid, Without,
};
use request::{delete_by_id, post, select, select_by_id, Request};
use constant::{LYRIC_EXTENSION, YAML_EXTENSION};

use tokio::task::{JoinError};

mod constant;
mod error;
mod fs;
mod io;
mod request;

#[derive(Clone)]
pub struct FileRepo {
    // join_handle: Arc<Pin<Box<dyn Future<Output = bool>>>>,
    tx: mpsc::Sender<Request>,
    path: String,
}

impl Debug for FileRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileRepo: {}", self.path)
    }
}

fn check_members(playlist: &Playlist, lyric_ids: &[Uuid]) -> impl futures::Future<Output = Result<(), FileRepoError>> {
    if let Some(member) = playlist.members.iter().find(|member| !lyric_ids.contains(member))
    {
        futures::future::ready(Err(FileRepoError::PlaylistInvalidMember(playlist.id.to_string(), member.to_string())))
    }
    else {
        futures::future::ready(Ok(()))
    }
}


async fn handle_request<P, Q>(request: Request, source_dir: String, lyric_path: P, playlist_path: Q) -> Result<(), ModelError> 
where P: Fn(&Uuid) -> PathBuf, Q: Fn(&Uuid) -> PathBuf
{
    match request {
        Request::Stop(sender) => {
            async {
                Ok::<(), FileRepoError>(())
            }
            .map(|v| sender.send(v))
            .map_err(|_| ModelError::SendFailed("Stop".to_string()))
            .await?;
            Err(ModelError::Stop)
        },
        Request::LyricSummaries(sender) => {
            io::get_list(
                &source_dir,
                LYRIC_EXTENSION,
                io::get_lyric_summary,
            )
            .map(|v|sender.send(v))
            .map_err(|_| ModelError::SendFailed("LyricSummaries".to_string()))
            .await
        }
        Request::LyricList(sender) => {
            io::get_list(
                &source_dir, 
                LYRIC_EXTENSION, 
                io::get_lyric,
            )
            .map(|v| sender.send(v))
            .map_err(|_| ModelError::SendFailed("LyricList".to_string()))
            .await
        }
        Request::LyricItem(uuid, sender) => {
            io::get_lyric(lyric_path(&uuid))
            .map(|v| sender.send(v))
            .map_err(|_| ModelError::SendFailed(format!("LyricItem {uuid}")))
            .await
        }
        Request::LyricDelete(uuid, sender) => {
            async {
                let playlists =
                    lyric_path(&uuid)
                    .remove()
                    .and_then(|_|
                        io::get_list(
                            &source_dir,
                            YAML_EXTENSION,
                            io::get_playlist
                        )
                    )
                    .await?;
                for mut playlist in playlists {
                    if playlist.members.contains(&uuid) {
                        playlist.members = playlist.members.without(&uuid);
                        io::post_item(
                            source_dir.full_path(&uuid.to_string(), YAML_EXTENSION),
                            playlist,
                        )
                        .await?;
                    }
                }
                Ok::<(), FileRepoError>(())
            }
            .map(|v| sender.send(v))
            .map_err(|_| ModelError::SendFailed(format!("LyricDelete {uuid}")))
            .await
        }
        Request::LyricPost(lyric, sender) => {
            let path = lyric_path(&lyric.id);
            io::post_item(
                &path,
                lyric,
            )
            .and_then(|_| io::get_lyric(&path))
            .map(|v| sender.send(v))
            .map_err(|e| ModelError::SendFailed(format!("LyricPost {}", e.unwrap().title)))
            .await
        }
        Request::PlaylistSummaries(sender) => {
            io::get_list(
                &source_dir,
                YAML_EXTENSION,
                io::get_playlist,
            )
            .map_ok(lipl_types::summaries)
            .map(|v| sender.send(v))
            .map_err(|_| ModelError::SendFailed("PlaylistSummaries".to_string()))
            .await
        }
        Request::PlaylistList(sender) => {
            io::get_list(
                &source_dir,
                YAML_EXTENSION,
                io::get_playlist
            )
            .map(|v| sender.send(v))
            .map_err(|_| ModelError::SendFailed("PlaylistList".to_string()))
            .await
        }
        Request::PlaylistItem(uuid, sender) => {
            io::get_playlist(playlist_path(&uuid))
            .map(|v| sender.send(v))
            .map_err(|_| ModelError::SendFailed(format!("PlaylistItem {uuid}")))
            .await
        }
        Request::PlaylistDelete(uuid, sender) => {
            let path = playlist_path(&uuid);
            path
            .remove()
            .map(|v| sender.send(v))
            .map_err(|_| ModelError::SendFailed(format!("PlaylistDelete {uuid}")))
            .await
        }
        Request::PlaylistPost(playlist, sender) => {
            io::get_list(
                &source_dir,
                LYRIC_EXTENSION,
                io::get_lyric_summary,
            )
            .map_ok(|summaries| lipl_types::ids(summaries.into_iter()))
            .and_then(|ids| check_members(&playlist, &ids))
            .and_then(
                |_| io::post_item(
                    playlist_path(&playlist.id),
                    playlist.clone(),
                )
            )
            .and_then(|_| io::get_playlist(
                    playlist_path(&playlist.id)
                )
            )
            .map(|v| sender.send(v))
            .map_err(|e| ModelError::SendFailed(format!("PlaylistPost {}", e.unwrap().title)))
            .await
        }
    }
}

impl FileRepo {
    pub fn new(
        source_dir: String,
    ) -> ModelResult<(FileRepo, impl Future<Output = Result<bool, JoinError>>)> {
        source_dir.is_dir().map_err(|_| ModelError::NoPath(source_dir.clone().into()))?;
        let dir = source_dir.clone();

        let (tx, rx) = mpsc::channel::<Request>(10);

        let join_handle = tokio::spawn(async move {
            let lyric_path = |uuid: &Uuid| source_dir.clone().full_path(&uuid.to_string(), LYRIC_EXTENSION);
            let playlist_path = |uuid: &Uuid| source_dir.clone().full_path(&uuid.to_string(), YAML_EXTENSION);

            rx
            .map(Ok)
            .try_for_each(|request| 
                handle_request(
                    request,
                    source_dir.clone(),
                    lyric_path,
                    playlist_path,
                )
            )
            .await
            .is_ok()
        });

        Ok(
            (
                FileRepo {
                    path: dir,
                    tx,
                },
                join_handle,
            )
        )
    }

}

#[async_trait]
impl LiplRepo for FileRepo {
    type Error = FileRepoError;

    #[tracing::instrument]
    async fn get_lyrics(&self) -> Result<Vec<Lyric>, Self::Error> {
        time_it!(
            select(&mut self.tx.clone(), Request::LyricList)    
        )
    }

    #[tracing::instrument]
    async fn get_lyric_summaries(&self) -> Result<Vec<Summary>, Self::Error> {
        time_it!(
            select(&mut self.tx.clone(), Request::LyricSummaries)
        )
    }

    #[tracing::instrument]
    async fn get_lyric(&self, id: Uuid) -> Result<Lyric, Self::Error> {
        time_it!(
            select_by_id(&mut self.tx.clone(), id, Request::LyricItem)
        )
    }

    #[tracing::instrument]
    async fn post_lyric(&self, lyric: Lyric) -> Result<Lyric, Self::Error> {
        time_it!(
            post(&mut self.tx.clone(), lyric, Request::LyricPost)
        )
    }

    #[tracing::instrument]
    async fn delete_lyric(&self, id: Uuid) -> Result<(), Self::Error> {
        time_it!(
            delete_by_id(&mut self.tx.clone(), id, Request::LyricDelete)
        )
    }

    #[tracing::instrument]
    async fn get_playlists(&self) -> Result<Vec<Playlist>, Self::Error> {
        time_it!(
            select(&mut self.tx.clone(), Request::PlaylistList)
        )
    }

    #[tracing::instrument]
    async fn get_playlist_summaries(&self) -> Result<Vec<Summary>, Self::Error> {
        time_it!(
            select(&mut self.tx.clone(), Request::PlaylistSummaries)
        )
    }

    #[tracing::instrument]
    async fn get_playlist(&self, id: Uuid) -> Result<Playlist, Self::Error> {
        time_it!(
            select_by_id(&mut self.tx.clone(), id, Request::PlaylistItem)
        )
    }

    #[tracing::instrument]
    async fn post_playlist(&self, playlist: Playlist) -> Result<Playlist, Self::Error> {
        time_it!(
            post(&mut self.tx.clone(), playlist, Request::PlaylistPost)
        )
    }

    #[tracing::instrument]
    async fn delete_playlist(&self, id: Uuid) -> Result<(), Self::Error> {
        time_it!(
            delete_by_id(&mut self.tx.clone(), id, Request::PlaylistDelete)
        )
    }

    #[tracing::instrument]
    async fn stop(&self) -> Result<(), Self::Error> {
        select(&mut self.tx.clone(), Request::Stop).await?;
        Ok::<(), FileRepoError>(())
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;


    #[test]
    fn file_repo_is_sized() {
        assert_eq!(size_of::<super::FileRepo>(), 48);
    }
}