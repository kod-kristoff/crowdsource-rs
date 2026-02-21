use sqlx::{Connection, PgPool};

use crowdsource::configuration::get_configuration;
use crowdsource::domain::crowdsrc::service::Service;
use crowdsource::inbound::http::HttpServer;
use crowdsource::inbound::http::HttpServerConfig;
use crowdsource::outbound::email_user_notifier::EmailUserNotifier;
use crowdsource::outbound::sqlx_user_repository::SqlxUserRepository;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // A minimal tracing middleware for request logging.
    tracing_subscriber::fmt::init();

    let config = get_configuration()?;

    let db_pool = PgPool::connect(&config.database.connection_string()).await?;

    let user_repo = SqlxUserRepository::new(db_pool);
    let user_notifier = EmailUserNotifier::new();
    let service = Service::new(user_repo, user_notifier);
    let http_config = HttpServerConfig { port: "3000" };
    let http_server = HttpServer::new(service, http_config).await?;
    http_server.run().await?;
    Ok(())
}
