use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    domain::crowdsrc::models::user::{CreateUserError, UserNameError},
    inbound::http::handlers::create_user::ParseCreateUserHttpRequestError,
};

#[derive(Debug, Clone)]
pub struct ApiSuccess<T: serde::Serialize + PartialEq>(StatusCode, Json<ApiResponseBody<T>>);

impl<T> PartialEq for ApiSuccess<T>
where
    T: serde::Serialize + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1.0 == other.1.0
    }
}

impl<T: serde::Serialize + PartialEq> ApiSuccess<T> {
    pub fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(ApiResponseBody::new(status, data)))
    }
}

impl<T: serde::Serialize + PartialEq> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::InternalServerError(e.to_string())
    }
}

impl From<CreateUserError> for ApiError {
    fn from(e: CreateUserError) -> Self {
        match e {
            CreateUserError::DuplicateUserName { username } => {
                Self::UnprocessableEntity(format!("user with username {} already exists", username))
            }
            CreateUserError::DuplicateEmail { email } => {
                Self::UnprocessableEntity(format!("user with email '{}' already exists", email))
            }
            CreateUserError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<ParseCreateUserHttpRequestError> for ApiError {
    fn from(e: ParseCreateUserHttpRequestError) -> Self {
        let message = match e {
            ParseCreateUserHttpRequestError::Name(UserNameError::Empty) => {
                "username can't be empty".to_string()
            }
            ParseCreateUserHttpRequestError::Name(UserNameError::WithWhitespace {
                invalid_username,
            }) => format!("username '{}' is not valid", invalid_username),
            ParseCreateUserHttpRequestError::EmailAddress(cause) => {
                format!("email address {} is invalid", cause.invalid_email)
            }
        };

        Self::UnprocessableEntity(message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        use ApiError::*;

        match self {
            InternalServerError(e) => {
                tracing::error!("{}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponseBody::new_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    )),
                )
                    .into_response()
            }
            UnprocessableEntity(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponseBody::new_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    message,
                )),
            )
                .into_response(),
        }
    }
}

/// Generic response structure shared by all API responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ApiResponseBody<T: serde::Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

impl<T: serde::Serialize + PartialEq> ApiResponseBody<T> {
    pub fn new(status_code: StatusCode, data: T) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data,
        }
    }
}

impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: ApiErrorData { message },
        }
    }
}

/// The response data format for all error responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ApiErrorData {
    pub message: String,
}
