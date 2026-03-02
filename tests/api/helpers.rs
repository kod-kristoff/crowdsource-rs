use std::{collections::HashMap, sync::Arc};

use crowdsource::{
    configuration::{DatabaseSettings, get_configuration},
    domain::crowdsrc::{models::user::EmailAddress, service::Service},
    inbound::http::HttpServer,
    outbound::{
        collecting_user_notifier::CollectingUserNotifier, sqlx_user_repository::SqlxUserRepository,
    },
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    user_email_map: Arc<RwLock<HashMap<EmailAddress, String>>>,
    address: String,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", dbg!(&self.address), path)
    }

    pub async fn post_users(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(self.url("/api/users"))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&configuration.database).await;
    let user_email_map = Arc::new(RwLock::new(HashMap::new()));
    let user_repo = SqlxUserRepository::new(db_pool.clone());
    let user_notifier = CollectingUserNotifier::new(user_email_map.clone());
    let crwdsrc_service = Service::new(user_repo, user_notifier);
    let config = crowdsource::inbound::http::HttpServerConfig { port: "0" };
    let server = HttpServer::new(crwdsrc_service, config).await.unwrap();
    let address = server.local_addr().unwrap();
    tokio::spawn(async move { server.run().await });
    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    TestApp {
        address: address.to_string(),
        user_email_map,
        db_pool,
        api_client,
    }
}
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: "password".to_string(),
        ..config.clone()
    };
    let mut connection = PgConnection::connect_with(&maintenance_settings.connection_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.connection_options())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
