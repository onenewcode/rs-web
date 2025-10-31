use axum_login::AuthUser;
use axum_login::AuthnBackend;
use axum_login::AuthzBackend;
use serde::Deserialize;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

#[derive(Deserialize)]
pub struct PageParams {
    pub page: Option<u64>,
    pub size: Option<u64>,
}
#[derive(Clone, Deserialize, Debug)]
pub struct User {
    id: i64,
    pub username: String,
    password: String,
}
impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes() // We use the password hash as the auth
        // hash--what this means
        // is when the user changes their password the
        // auth session becomes invalid.
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: i32,
}

impl Backend {
    pub fn new(db: i32) -> Self {
        Self { db }
    }
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = std::fmt::Error;

    async fn authenticate(
        &self,
        _creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        //     let user: Option<Self::User> = sqlx::query_as("select * from users where username = ? ")
        //         .bind(creds.username)
        //         .fetch_optional(&self.db)
        //         .await?;

        //     // Verifying the password is blocking and potentially slow, so we'll do so via
        //     // `spawn_blocking`.
        //     task::spawn_blocking(|| {
        //         // We're using password-based authentication: this works by comparing our form
        //         // input with an argon2 password hash.
        //         Ok(user.filter(|user| verify_password(creds.password, &user.password).is_ok()))
        //     })
        //     .await?
        // }

        // async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        //     let user = sqlx::query_as("select * from users where id = ?")
        //         .bind(user_id)
        //         .fetch_optional(&self.db)
        //         .await?;

        //     Ok(user)
        todo!()
    }
    
    #[allow(refining_impl_trait)]
    fn get_user(
        &self,
        _user_id: &axum_login::UserId<Self>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Self::User>, Self::Error>> + Send>> {
        Box::pin(async move {
            Ok(None)  // 返回一个空的用户结果
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Permission {
    pub name: String,
}

impl From<&str> for Permission {
    fn from(name: &str) -> Self {
        Permission {
            name: name.to_string(),
        }
    }
}

impl AuthzBackend for Backend {
    type Permission = Permission;

    async fn get_group_permissions(
        &self,
        _user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        // let permissions: Vec<Self::Permission> = sqlx::query_as(
        //     r#"
        //     select distinct permissions.name
        //     from users
        //     join users_groups on users.id = users_groups.user_id
        //     join groups_permissions on users_groups.group_id = groups_permissions.group_id
        //     join permissions on groups_permissions.permission_id = permissions.id
        //     where users.id = ?
        //     "#,
        // )
        // .bind(user.id)
        // .fetch_all(&self.db)
        // .await?;

        // Ok(permissions.into_iter().collect())
        todo!()
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
