mod v1;

use axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let app = Router::new();

    let listener = TcpListener::bind("localhost:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
