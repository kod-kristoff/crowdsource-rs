use crate::domain::crowdsrc::ports::UserNotifier;

#[derive(Debug, Clone)]
pub struct EmailUserNotifier {}

impl EmailUserNotifier {
    pub fn new() -> Self {
        Self {}
    }
}

impl UserNotifier for EmailUserNotifier {
    #[allow(clippy::manual_async_fn)]
    fn user_created(
        &self,
        _user: &crate::domain::crowdsrc::models::user::User,
    ) -> impl Future<Output = ()> + Send {
        async {}
    }
}
