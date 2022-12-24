use async_trait::async_trait;
use lipl_core::{Lyric, LyricPost, Playlist, PlaylistPost, Summary, Uuid};
use rest_api_client::{ApiClient, ApiRequest};

use crate::{UploadResult};

#[async_trait]
pub trait Api {
    async fn lyric_summaries(&self) -> UploadResult<Vec<Summary>>;
    async fn lyric_delete(&self, id: Uuid) -> UploadResult<()>;
    async fn lyric_insert(&self, lyric_post: LyricPost) -> UploadResult<Lyric>;
    async fn playlist_summaries(&self) -> UploadResult<Vec<Summary>>;
    async fn playlist_delete(&self, id: Uuid) -> UploadResult<()>;
    async fn playlist_insert(&self, playlist_post: PlaylistPost) -> UploadResult<Playlist>;
}

pub struct UploadClient {
    inner: ApiClient,
}

impl From<ApiClient> for UploadClient {
    fn from(api_client: ApiClient) -> Self {
        Self {
            inner: api_client
        }
    }
}

#[async_trait]
impl Api for UploadClient {
    async fn lyric_summaries(&self) -> UploadResult<Vec<Summary>> {
        self.inner.get("lyric")
        .await
        .map_err(Into::into)
    }

    async fn lyric_delete(&self, id: Uuid) -> UploadResult<()> {
        self.inner.delete(&format!("lyric/{}", id))
        .await
        .map_err(Into::into)
    }

    async fn lyric_insert(&self, lyric_post: LyricPost) -> UploadResult<Lyric> {
        self.inner.post("lyric", lyric_post)
        .await
        .map_err(Into::into)
    }

    async fn playlist_summaries(&self) -> UploadResult<Vec<Summary>> {
        self.inner.get("playlist")
        .await
        .map_err(Into::into)
    }

    async fn playlist_delete(&self, id: Uuid) -> UploadResult<()> {
        self.inner.delete(&format!("playlist/{}", id))
        .await
        .map_err(Into::into)
    }

    async fn playlist_insert(&self, playlist_post: PlaylistPost) -> UploadResult<Playlist> {
        self.inner.post("playlist", playlist_post)
        .await
        .map_err(Into::into)
    }
}