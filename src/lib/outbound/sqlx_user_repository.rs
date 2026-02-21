use sqlx::PgPool;

use crate::domain::crowdsrc::ports::UserRepository;

#[derive(Debug, Clone)]
pub struct SqlxUserRepository {
    db_pool: PgPool,
}

impl SqlxUserRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

impl UserRepository for SqlxUserRepository {
    #[allow(clippy::manual_async_fn)]
    fn create_user(
        &self,
        req: &crate::domain::crowdsrc::models::user::CreateUserRequest,
    ) -> impl Future<
        Output = Result<
            crate::domain::crowdsrc::models::user::User,
            crate::domain::crowdsrc::models::user::CreateUserError,
        >,
    > + Send {
        async { todo!() }
    }
}
