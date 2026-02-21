use axum::{Json, extract::State, http::StatusCode};

use crate::{
    domain::crowdsrc::{
        models::user::{
            CreateUserRequest, EmailAddress, EmailAddressError, User, UserName, UserNameError,
        },
        ports::CrowdSrcService,
    },
    inbound::http::{
        AppState,
        responses::{ApiError, ApiSuccess},
    },
};

/// Create a new [User].
///
/// # Responses
///
/// - 201 Created: the [User] was successfully created.
/// - 422 Unprocessable entity: An [User] with the same name already exists.
pub async fn create_user<CS: CrowdSrcService>(
    State(state): State<AppState<CS>>,
    Json(body): Json<CreateUserHttpRequestBody>,
) -> Result<ApiSuccess<CreateUserResponseData>, ApiError> {
    let domain_req = body.try_into_domain()?;
    state
        .crwdsrc_service
        .create_user(&domain_req)
        .await
        .map_err(ApiError::from)
        .map(|ref user| ApiSuccess::new(StatusCode::CREATED, user.into()))
}

/// The body of an [User] creation request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct CreateUserHttpRequestBody {
    username: String,
    email_address: String,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseCreateUserHttpRequestError {
    #[error(transparent)]
    Name(#[from] UserNameError),
    #[error(transparent)]
    EmailAddress(#[from] EmailAddressError),
}

impl CreateUserHttpRequestBody {
    /// Converts the HTTP request body into a domain request.
    fn try_into_domain(self) -> Result<CreateUserRequest, ParseCreateUserHttpRequestError> {
        let name = UserName::new(&self.username)?;
        let email = EmailAddress::new(&self.email_address)?;
        Ok(CreateUserRequest::new(name, email))
    }
}

/// The response body data field for successful [User] creation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CreateUserResponseData {
    id: String,
}

impl From<&User> for CreateUserResponseData {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem;
    use std::sync::Arc;

    use anyhow::anyhow;
    use uuid::Uuid;

    use crate::domain::crowdsrc::models::user::CreateUserError;
    use crate::domain::crowdsrc::models::user::CreateUserRequest;
    use crate::domain::crowdsrc::models::user::User;
    use crate::domain::crowdsrc::ports::CrowdSrcService;

    use super::*;

    #[derive(Clone)]
    struct MockCrowdSrcService {
        create_user_result: Arc<std::sync::Mutex<Result<User, CreateUserError>>>,
    }

    impl CrowdSrcService for MockCrowdSrcService {
        async fn create_user(&self, _: &CreateUserRequest) -> Result<User, CreateUserError> {
            let mut guard = self.create_user_result.lock();
            let mut result = Err(CreateUserError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.as_deref_mut().unwrap(), &mut result);
            result
        }
    }

    async fn run_create_user(
        service: MockCrowdSrcService,
        user_name: &UserName,
        user_email: &EmailAddress,
    ) -> Result<ApiSuccess<CreateUserResponseData>, ApiError> {
        let state = axum::extract::State(AppState {
            crwdsrc_service: Arc::new(service),
        });
        let body = axum::extract::Json(CreateUserHttpRequestBody {
            username: user_name.to_string(),
            email_address: user_email.to_string(),
        });
        create_user(state, body).await
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_user_fails_if_email_exists() {
        let user_name = UserName::new("Kristoffer").unwrap();
        let user_email = EmailAddress::new("kristoffer@example.com").unwrap();
        let service = MockCrowdSrcService {
            create_user_result: Arc::new(std::sync::Mutex::new(Err(
                CreateUserError::DuplicateEmail {
                    email: user_email.clone(),
                },
            ))),
        };

        let actual = run_create_user(service, &user_name, &user_email).await;

        assert!(
            actual.is_err(),
            "expected create_user to fail, but got {:?}",
            actual
        );

        let expected_err =
            ApiError::UnprocessableEntity(format!("user with email '{user_email}' already exists"));
        let actual_err = actual.unwrap_err();
        assert_eq!(
            actual_err, expected_err,
            "expected ApiError {:?}, but got {:?}",
            expected_err, actual_err
        )
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_user_fails_if_username_exists() {
        let user_name = UserName::new("Kristoffer").unwrap();
        let user_email = EmailAddress::new("kristoffer@example.com").unwrap();
        let service = MockCrowdSrcService {
            create_user_result: Arc::new(std::sync::Mutex::new(Err(
                CreateUserError::DuplicateUserName {
                    username: user_name.clone(),
                },
            ))),
        };

        let actual = run_create_user(service, &user_name, &user_email).await;

        assert!(
            actual.is_err(),
            "expected create_user to fail, but got {:?}",
            actual
        );

        let expected_err =
            ApiError::UnprocessableEntity(format!("user with username {user_name} already exists"));
        let actual_err = actual.unwrap_err();
        assert_eq!(
            actual_err, expected_err,
            "expected ApiError {:?}, but got {:?}",
            expected_err, actual_err
        )
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_user_success() {
        let user_name = UserName::new("Kristoffer").unwrap();
        let user_email = EmailAddress::new("kristoffer@example.com").unwrap();
        let user_id = Uuid::new_v4();
        let service = MockCrowdSrcService {
            create_user_result: Arc::new(std::sync::Mutex::new(Ok(User::new(
                user_id,
                user_name.clone(),
                user_email.clone(),
            )))),
        };

        let actual = run_create_user(service, &user_name, &user_email).await;

        assert!(
            actual.is_ok(),
            "expected create_user to succeed, but got {:?}",
            actual
        );
        let expected = ApiSuccess::new(
            StatusCode::CREATED,
            CreateUserResponseData {
                id: user_id.to_string(),
            },
        );
        let actual = actual.unwrap();
        assert_eq!(
            actual, expected,
            "expected ApiSuccess {:?}, but got {:?}",
            expected, actual
        )
    }
}
