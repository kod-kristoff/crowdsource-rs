use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Transaction};
use uuid::Uuid;

use crate::domain::crowdsrc::{
    models::user::{CreateUserError, EmailAddress, User, UserName},
    ports::UserRepository,
};

#[derive(Debug, Clone)]
pub struct SqlxUserRepository {
    db_pool: PgPool,
}

impl SqlxUserRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    async fn save_user(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        username: &UserName,
        email: &EmailAddress,
    ) -> Result<(Uuid, DateTime<Utc>), sqlx::Error> {
        let id = Uuid::new_v4();
        let username = username.to_string();
        let email = email.to_string();
        let created_at = Utc::now();
        let query = sqlx::query!(
            "INSERT INTO users (id, email, username, created_at) VALUES ($1, $2, $3, $4)",
            id,
            email,
            username,
            created_at,
        );
        tx.execute(query).await?;
        Ok((id, created_at))
    }
}

impl UserRepository for SqlxUserRepository {
    #[allow(clippy::manual_async_fn)]
    async fn create_user(
        &self,
        req: &crate::domain::crowdsrc::models::user::CreateUserRequest,
    ) -> Result<
        crate::domain::crowdsrc::models::user::User,
        crate::domain::crowdsrc::models::user::CreateUserError,
    > {
        let mut tx = self
            .db_pool
            .begin()
            .await
            .context("failed to start Postgres transaction")?;

        let (user_id, created_at) = self
            .save_user(&mut tx, req.username(), req.email())
            .await
            .map_err(|e| {
                if is_unique_constraint_violation(&e) {
                    CreateUserError::DuplicateEmail {
                        email: req.email().clone(),
                    }
                } else {
                    anyhow::anyhow!(e)
                        .context(format!(
                            "failed to save user with username {:?} and email {:?}",
                            req.username(),
                            req.email(),
                        ))
                        .into()
                }
            })?;

        tx.commit()
            .await
            .context("failed to commit Postgres transaction")?;

        Ok(User::new(
            user_id,
            req.username().clone(),
            req.email().clone(),
            created_at,
        ))
    }
}

const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "2067";

#[allow(clippy::collapsible_if)]
fn is_unique_constraint_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            if code == UNIQUE_CONSTRAINT_VIOLATION_CODE {
                dbg!(db_err);
                return true;
            }
        }
    }

    false
}
