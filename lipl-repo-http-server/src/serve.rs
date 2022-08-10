use anyhow::Result;
use lipl_types::LiplRepo;
use tokio::sync::oneshot;
use tokio::signal;
use tracing::{info, error};
use warp::Filter;

use crate::constant;
use crate::message;
use crate::param;
use crate::filter::{get_lyric_routes, get_playlist_routes};
use crate::param::DbType;

async fn run<T: LiplRepo<E> + 'static, E: std::error::Error + 'static>(repo: T, port: u16) -> Result<()> {
    let (tx, rx) = oneshot::channel::<()>();
    let signals = signal::ctrl_c();
    
    tokio::task::spawn(async move {
        signals.await
        .map(|_| tx.send(()))
    });

    let routes = 
        get_lyric_routes(repo.clone(), constant::LYRIC)
        .or(
            get_playlist_routes(repo.clone(), constant::PLAYLIST)
        )
        .with(warp::trace::request());

    let (_address, server) = 
        warp::serve(routes)
        .try_bind_with_graceful_shutdown((constant::HOST, port), async move {
            rx.await.ok();
            info!("{}", message::STOPPING);
            if let Err(error) = repo.stop().await {
                error!("{}", error);
            };
            info!("{}", message::BACKUP_COMPLETE);
        })?;

    server.await;

    Ok(())

}

pub async fn serve(param: param::Serve) -> Result<()> {

    match param.source.parse::<DbType>()? {
        DbType::File(_, file) => {
            let repo = file.await?;
            run(repo, param.port).await?;

        },
        DbType::Postgres(_, postgres) => {
            let repo = postgres.await?;
            run(repo, param.port).await?;
        }
    }
    Ok(())
}