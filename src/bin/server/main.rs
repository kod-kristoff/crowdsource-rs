use crowdsource::inbound::http::{HttpServer, HttpServerConfig};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // A minimal tracing middleware for request logging.
    tracing_subscriber::fmt::init();

    let http_config = HttpServerConfig { port: "3000" };
    let http_server = HttpServer::new(http_config).await?;
    http_server.run().await?;
    Ok(())
}
