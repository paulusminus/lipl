use crate::param::{ListCommand, CopyCommand, DbType};
use anyhow::Result;
use lipl_core::{LiplRepo, RepoDb};
use tracing::{info};

pub async fn list<E: std::error::Error>(repo: impl LiplRepo<E>, yaml: bool) -> std::result::Result<(), E> {

    let db = RepoDb {
        lyrics: repo.get_lyrics().await?,
        playlists: repo.get_playlists().await?,
    };

    println!("{}", if yaml { db.to_yaml().unwrap() } else { db.to_string() }) ;
    Ok(())
}

pub async fn repo_list(args: ListCommand) -> Result<()> {

    match args.source {
        DbType::File(_, f) => {
            list(f.await?, args.yaml).await?;
        },
        DbType::Postgres(_, f) => {
            list(f.await?, args.yaml).await?;
        }
    }

    Ok(())
}

pub async fn copy<E, F>(source: impl LiplRepo<E>, target: impl LiplRepo<F>) -> anyhow::Result<()> 
where 
    E: std::error::Error + Send + Sync + 'static, 
    F: std::error::Error + Send + Sync + 'static, 
{
    for lyric in source.get_lyrics().await? {
        info!("Copying lyric {} with id {}", lyric.title, lyric.id);
        target.post_lyric(lyric).await.unwrap();
    }

    for playlist in source.get_playlists().await? {
        info!("Copying playlist {} with id {}", playlist.title, playlist.id);
        target.post_playlist(playlist).await.unwrap();
    }

    Ok(())
}


pub async fn repo_copy(args: CopyCommand) -> Result<()> {
    info!(
        "Start copying {:?} to {:?}",
        &args.source,
        &args.target,
    );

    // let source_db_type = args.source.parse::<DbType>()?;
    // let target_db_type = args.target.parse::<DbType>()?;

    match args.source {
        DbType::File(_, source_file) => {
            match args.target {
                DbType::File(_, target_file) => {
                    copy(source_file.await?, target_file.await?).await?;
                },
                DbType::Postgres(_, target_postgres) => {
                    copy(source_file.await?, target_postgres.await?).await?;
                }
            }
        },
        DbType::Postgres(_, source_postgres) => {
            match args.target {
                DbType::File(_, target_file) => {
                    copy(source_postgres.await?, target_file.await?).await?;
                },
                DbType::Postgres(_, target_postgres) => {
                    copy(source_postgres.await?, target_postgres.await?).await?;
                }
            }
        },
    }

     Ok(())

}
