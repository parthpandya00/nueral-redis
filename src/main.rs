use tokio::net::TcpListener;
use anyhow::Result;
use std::sync::Arc;
use neural_redis::{db, connection};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:6379").await?;
    let db = Arc::new(db::DB::new());
    println!("Listening on port 6379");

    loop {
        let (socket, _) = listener.accept().await?;
        let db_clone = Arc::clone(&db);
        tokio::spawn(async move {
            connection::handle_connection(socket, db_clone).await;
        });
    }
}