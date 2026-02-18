use std::{collections::HashMap, sync::Arc};

use crowdsource::{
    domain::crowdsrc::{models::user::EmailAddress, service::Service},
    inbound::http::HttpServer,
    outbound::collecting_user_notifier::CollectingUserNotifier,
};
use tokio::sync::RwLock;

pub struct TestApp {
    user_email_map: Arc<RwLock<HashMap<EmailAddress, String>>>,
    address: String,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let user_email_map = Arc::new(RwLock::new(HashMap::new()));
        let user_repo = todo!();
        let user_notifier = CollectingUserNotifier::new(user_email_map.clone());
        let crwdsrc_service = Service::new(user_repo, user_notifier);
        let config = crowdsource::inbound::http::HttpServerConfig { port: "0" };
        let server = HttpServer::new(crwdsrc_service, config).await.unwrap();
        let address = server.local_addr().unwrap();
        tokio::spawn(async move { server.run().await });
        TestApp {
            address: address.to_string(),
            user_email_map,
        }
    }
}
