use std::{sync::Arc, time::Duration};

use diesel_async::AsyncPgConnection;
use tokio::time::interval;

use crate::{
    database::{actions, DbError},
    files::delete_file,
    DbPool,
};

async fn delete_expired_files(conn: &mut AsyncPgConnection) -> Result<(), DbError> {
    let files = actions::get_expired_files(conn).await?;
    for file in files {
        let _ = delete_file(conn, &file).await;
    }

    Ok(())
}

pub async fn cleanup_job(conn_pool: Arc<DbPool>) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_secs(3600)); // 1 hour

    loop {
        interval.tick().await;

        let mut conn = conn_pool.get().await.expect("Failed to get connection");

        if let Err(e) = delete_expired_files(&mut *conn).await {
            eprintln!("Failed to delete expired files: {}", e);
        }
    }
}
