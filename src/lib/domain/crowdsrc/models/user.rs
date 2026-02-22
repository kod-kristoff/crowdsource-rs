use std::{fmt, str::FromStr};

#[derive(Debug, Clone)]
pub struct User {
    id: uuid::Uuid,
    username: UserName,
    email_addr: EmailAddress,
}

impl User {
    pub fn new(id: uuid::Uuid, username: UserName, email_addr: EmailAddress) -> Self {
        Self {
            id,
            username,
            email_addr,
        }
    }

    pub fn id(&self) -> &uuid::Uuid {
        &self.id
    }

    pub fn username(&self) -> &UserName {
        &self.username
    }

    pub fn email(&self) -> &EmailAddress {
        &self.email_addr
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A valid email address.
pub struct EmailAddress(email_address::EmailAddress);

#[derive(Debug, Clone, thiserror::Error)]
#[error("'{invalid_email}' is not a valid email address: {message}")]
pub struct EmailAddressError {
    pub invalid_email: String,
    pub message: String,
}

impl EmailAddress {
    pub fn new(email: &str) -> Result<Self, EmailAddressError> {
        let email =
            email_address::EmailAddress::from_str(email).map_err(|err| EmailAddressError {
                invalid_email: email.to_string(),
                message: err.to_string(),
            })?;
        Ok(Self(email))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserName(String);

#[derive(Debug, Clone, thiserror::Error)]
pub enum UserNameError {
    #[error("username cannot be empty")]
    Empty,
    #[error("username cannot contain whitespace: '{invalid_username}'")]
    WithWhitespace { invalid_username: String },
}

impl UserName {
    pub fn new(raw: &str) -> Result<Self, UserNameError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(UserNameError::Empty)
        } else if trimmed.contains(|c: char| c.is_whitespace()) {
            Err(UserNameError::WithWhitespace {
                invalid_username: raw.to_string(),
            })
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }
}

impl fmt::Display for UserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The fields required by the domain to create an [User].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateUserRequest {
    name: UserName,
    email: EmailAddress,
}

impl CreateUserRequest {
    pub fn new(name: UserName, email: EmailAddress) -> Self {
        Self { name, email }
    }

    pub fn name(&self) -> &UserName {
        &self.name
    }

    pub fn email(&self) -> &EmailAddress {
        &self.email
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error("user with user name {username} already exists")]
    DuplicateUserName { username: UserName },
    #[error("user with email {email} already exists")]
    DuplicateEmail { email: EmailAddress },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
}
