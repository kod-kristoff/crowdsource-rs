use crate::domain::crowdsrc::ports::UserRepository;

#[derive(Debug, Clone)]
pub struct SqlxUserRepository {}

impl SqlxUserRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl UserRepository for SqlxUserRepository {
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
