/*
   Module `ports` specifies the API by which external modules interact with the crowdsrc domain.

   All traits are bounded by `Send + Sync + 'static`, since their implementations must be shareable
   between request-handling threads.

   Trait methods are explicitly asynchronous, including `Send` bounds on response types,
   since the application is expected to always run in a multithreaded environment.
*/

use std::future::Future;

use crate::domain::crowdsrc::models::user::CreateUserError;
#[allow(unused_imports)] // UserName is used in doc comments
use crate::domain::crowdsrc::models::user::UserName;
use crate::domain::crowdsrc::models::user::{CreateUserRequest, User};

/// `CrowdSrcService` is the public API for the crowdsrc domain.
///
/// External modules must conform to this contract – the domain is not concerned with the
/// implementation details or underlying technology of any external code.
pub trait CrowdSrcService: Clone + Send + Sync + 'static {
    /// Asynchronously create a new [User].
    ///
    /// # Errors
    ///
    /// - [CreateUserError::Duplicate] if an [User] with the same [UserName] already exists.
    fn create_user(
        &self,
        req: &CreateUserRequest,
    ) -> impl Future<Output = Result<User, CreateUserError>> + Send;
}

/// `UserRepository` represents a store of user data.
///
/// External modules must conform to this contract – the domain is not concerned with the
/// implementation details or underlying technology of any external code.
pub trait UserRepository: Send + Sync + Clone + 'static {
    /// Asynchronously persist a new [User].
    ///
    /// # Errors
    ///
    /// - MUST return [CreateUserError::Duplicate] if an [User] with the same [UserName]
    ///   already exists.
    fn create_user(
        &self,
        req: &CreateUserRequest,
    ) -> impl Future<Output = Result<User, CreateUserError>> + Send;
}

/// `UserNotifier` triggers notifications to users.
///
/// Whether or the notification medium (email, SMS, etc.) is known by the business logic is a
/// judgement call based on your use case.
///
/// Some domains will always require email, for example, so hiding this detail would be
/// pointless.
///
/// For others, code coordinating notifications will be complex enough to warrant its own domain.
/// In this case, an `UserNotifier` adapter will call that domain's `Service`.
pub trait UserNotifier: Send + Sync + Clone + 'static {
    fn user_created(&self, user: &User) -> impl Future<Output = ()> + Send;
}
