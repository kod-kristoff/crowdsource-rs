use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::domain::crowdsrc::{models::user::EmailAddress, ports::UserNotifier};
#[derive(Clone, Debug)]
pub struct CollectingUserNotifier {
    user_email_map: Arc<RwLock<HashMap<EmailAddress, String>>>,
}

impl CollectingUserNotifier {
    pub fn new(user_email_map: Arc<RwLock<HashMap<EmailAddress, String>>>) -> Self {
        Self { user_email_map }
    }
}

impl UserNotifier for CollectingUserNotifier {
    #[allow(clippy::manual_async_fn)]
    fn user_created(
        &self,
        user: &crate::domain::crowdsrc::models::user::User,
    ) -> impl Future<Output = ()> + Send {
        async {
            self.user_email_map
                .write()
                .await
                .insert(user.email().clone(), String::new());
        }
    }
}
