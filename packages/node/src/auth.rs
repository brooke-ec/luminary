//! Handles authentication for the Luminary Node, including session management and user authentication.

use std::fmt::Debug;

use axum_login::{
    AuthManagerLayer, AuthManagerLayerBuilder, AuthUser, AuthnBackend,
    tower_sessions::{ExpiredDeletion, SessionManagerLayer, cookie},
};
use eyre::{Context, Result};
use password_auth::verify_password;
use sqlx::{SqlitePool, prelude::FromRow};
use tower_sessions_sqlx_store::SqliteStore;
use uuid::Uuid;

/// Acts as the authentication backend for the Luminary Node, handling user authentication and session management.
#[derive(Debug, Clone)]
pub struct LuminaryAuthentication {
    pool: SqlitePool,
}

impl LuminaryAuthentication {
    pub async fn setup(pool: SqlitePool) -> Result<AuthManagerLayer<LuminaryAuthentication, SqliteStore>> {
        // Set up session store and start background task to delete expired sessions
        let session_store = SqliteStore::new(pool.clone());
        session_store
            .migrate()
            .await
            .wrap_err("Failed to migrate sessions table")?;

        tokio::spawn(
            session_store
                .clone()
                .continuously_delete_expired(std::time::Duration::from_secs(60)),
        );

        // Set up session layer
        let session_layer = SessionManagerLayer::new(session_store)
            .with_expiry(axum_login::tower_sessions::Expiry::OnInactivity(
                cookie::time::Duration::minutes(30),
            ))
            .with_secure(false);

        // Auth service
        let backend = Self { pool: pool };
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        return Ok(auth_layer);
    }
}

// Taken from https://github.com/maxcountryman/axum-login/blob/main/examples/sqlite/src/users.rs
impl AuthnBackend for LuminaryAuthentication {
    type Credentials = LuminaryUserCredentials;
    type User = LuminaryUser;
    type Error = AuthError;

    async fn authenticate(&self, credentials: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as("select * from users where username = ? ")
            .bind(credentials.username)
            .fetch_optional(&self.pool)
            .await?;

        // Verifying the password is blocking and potentially slow, so we'll do so via
        // `spawn_blocking`.
        tokio::task::spawn_blocking(|| {
            // We're using password-based authentication--this works by comparing our form
            // input with an argon2 password hash.
            Ok(user.filter(|user| verify_password(credentials.password, &user.password).is_ok()))
        })
        .await?
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("select * from users where id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }
}

// I really don't like that I have to define this error type, but axum-login can't use eyre's error type
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TaskJoin(#[from] tokio::task::JoinError),
}

#[derive(Debug, Clone)]
pub struct LuminaryUserCredentials {
    username: String,
    password: String,
}

#[derive(Clone, FromRow)]
pub struct LuminaryUser {
    id: Uuid,
    username: String,
    password: String,
}

impl Debug for LuminaryUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuminaryUser")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
}

impl AuthUser for LuminaryUser {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        return self.id;
    }

    fn session_auth_hash(&self) -> &[u8] {
        return self.password.as_bytes();
    }
}
