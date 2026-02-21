/*!
   Module `service` provides the canonical implementation of the [CrowdSrcService] port. All
   crowdsrc-domain logic is defined here.
*/

use crate::domain::crowdsrc::models::user::CreateUserError;
use crate::domain::crowdsrc::models::user::{CreateUserRequest, User};
use crate::domain::crowdsrc::ports::{CrowdSrcService, UserNotifier, UserRepository};

/// Canonical implementation of the [CrowdSrcService] port, through which the crowdsrc domain API is
/// consumed.
#[derive(Debug, Clone)]
pub struct Service<R, N>
where
    R: UserRepository,
    N: UserNotifier,
{
    user_repo: R,
    user_notifier: N,
}

impl<R, N> Service<R, N>
where
    R: UserRepository,
    N: UserNotifier,
{
    pub fn new(user_repo: R, user_notifier: N) -> Self {
        Self {
            user_repo,
            user_notifier,
        }
    }
}

impl<R, N> CrowdSrcService for Service<R, N>
where
    R: UserRepository,
    N: UserNotifier,
{
    /// Create the [User] specified in `req` and trigger notifications.
    ///
    /// # Errors
    ///
    /// - Propagates any [CreateUserError] returned by the [UserRepository].
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, CreateUserError> {
        let result = self.user_repo.create_user(req).await;
        if let Ok(user) = result.as_ref() {
            self.user_notifier.user_created(user).await;
        }

        result
    }
}
