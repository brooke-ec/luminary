//! Handles authentication for the Luminary Node, including bearer token management and user authentication.

use std::fmt::Debug;

use eyre::{Context, ContextCompat, Result};
use log::error;
use luminary_macros::wrap_err;
use password_auth::verify_password;
use rand_chacha::{
    ChaCha12Rng,
    rand_core::{RngCore, SeedableRng},
};
use salvo::{
    oapi::extract::{JsonBody, PathParam},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::FromRow};
use uuid::Uuid;

use crate::{api::response::LuminaryResponse, eyre_fmt, obtain};

/// Returns a router containing all authentication-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/auth")
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("logout").post(logout))
        .push(
            Router::with_path("reset/{token}")
                .get(verify_reset_token)
                .post(reset_password),
        )
        .push(
            Router::with_hoop(protected).push(
                Router::with_path("users")
                    .get(get_users)
                    .post(create_user)
                    .push(Router::with_path("{user}").delete(delete_user)),
            ),
        );
}

/// Reads username and password from the request body, and returns an authentication token if the credentials are valid.
#[endpoint]
async fn login(depot: &mut Depot, body: JsonBody<LuminaryUserCredentials>) -> LuminaryResponse<String> {
    let auth = obtain!(depot, LuminaryAuthentication);

    let token = auth
        .login(body.into_inner())
        .await?
        .wrap_err("Invalid username or password")?;
    return Ok(token.into());
}

/// Logs out the current user, invalidating their authentication token.
#[endpoint]
async fn logout(req: &mut Request, depot: &mut Depot) -> LuminaryResponse<()> {
    let auth = obtain!(depot, LuminaryAuthentication);
    let token = extract_token(req).wrap_err("Missing or invalid authorization token")?;

    auth.logout(token).await?;
    return Ok(().into());
}

/// Fetches a list of all users.
#[endpoint]
async fn get_users(depot: &mut Depot) -> LuminaryResponse<Vec<LuminaryUser>> {
    let auth = obtain!(depot, LuminaryAuthentication);
    let users = auth.get_users().await?;
    return Ok(users.into());
}

/// Creates a new user with the given username, returning a reset token that can be used to set the user's password.
#[endpoint]
async fn create_user(depot: &mut Depot, body: JsonBody<CreateUserRequest>) -> LuminaryResponse<String> {
    let auth = obtain!(depot, LuminaryAuthentication);

    let reset_token = auth.create_user(&body.username).await?;
    return Ok(reset_token.into());
}

/// Request body for creating a new user.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    username: String,
}

/// Deletes the user with the given UUID.
#[endpoint]
async fn delete_user(depot: &mut Depot, user: PathParam<String>) -> LuminaryResponse<()> {
    let auth = obtain!(depot, LuminaryAuthentication);

    auth.delete_user(&user).await?;
    return Ok(().into());
}

/// Verifies that a reset token is valid.
#[endpoint]
async fn verify_reset_token(depot: &mut Depot, token: PathParam<String>) -> LuminaryResponse<String> {
    let auth = obtain!(depot, LuminaryAuthentication);

    let user = auth
        .get_user_from_reset_token(&token)
        .await?
        .wrap_err("Invalid reset token")?;

    return Ok(user.username.into());
}

/// Resets a user's password using a reset token, which is invalidated after use.
#[endpoint]
async fn reset_password(
    depot: &mut Depot,
    token: PathParam<String>,
    body: JsonBody<ResetPasswordRequest>,
) -> LuminaryResponse<()> {
    let auth = obtain!(depot, LuminaryAuthentication);

    let user = auth
        .get_user_from_reset_token(&token)
        .await?
        .wrap_err("Invalid reset token")?;

    auth.set_password(&user.uuid, body.password.clone()).await?;
    return Ok(().into());
}

/// Request body for resetting a user's password.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    password: String,
}

/// Acts as the authentication backend for the Luminary Node, handling user authentication and bearer token management.
#[derive(Debug, Clone)]
pub struct LuminaryAuthentication {
    pool: SqlitePool,
}

impl LuminaryAuthentication {
    /// Creates a new authentication handler backed by the given database pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn verify_password(
        &self,
        credentials: LuminaryUserCredentials,
    ) -> Result<Option<LuminaryUser>> {
        let user = sqlx::query_as!(
            LuminaryUser,
            "SELECT * FROM [user] WHERE [username] = ?",
            credentials.username
        )
        .fetch_optional(&self.pool)
        .await
        .wrap_err("Failed to query for user")?;

        // Verifying the password is blocking and pretty slow (~600ms), so run on a separate thread
        let user = tokio::task::spawn_blocking(move || {
            user.filter(|user| {
                user.password
                    .as_ref()
                    .is_some_and(|password| verify_password(&credentials.password, &password).is_ok())
            })
        })
        .await
        .wrap_err("Password verification task failed")?;

        return Ok(user);
    }

    /// Authenticates a user with the given credentials and returns a bearer token on success.
    pub async fn login(&self, credentials: LuminaryUserCredentials) -> Result<Option<String>> {
        // Terminate early if the user doesn't exist or the password is wrong
        let user = match self.verify_password(credentials).await? {
            None => return Ok(None),
            Some(u) => u,
        };

        let token = generate_token();

        // Store the token in the database, associated with the user
        let uuid = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO [session] ([uuid], [token], [user_agent], [user]) VALUES (?, ?, ?, ?)",
            uuid,
            token,
            "todo", // todo: capture user agent from request and store it here
            user.uuid
        )
        .execute(&self.pool)
        .await
        .wrap_err("Failed to create session")?;

        return Ok(Some(token));
    }

    /// Logs a token out by deleting it from the database, invalidating it
    pub async fn logout(&self, token: &str) -> Result<()> {
        sqlx::query!("DELETE FROM [session] WHERE [token] = ?", token)
            .execute(&self.pool)
            .await
            .wrap_err("Failed to delete session")?;

        return Ok(());
    }

    /// Find a user from their bearer token, or [None] if the token is invalid.
    async fn get_user_by_token(&self, token: &str) -> Result<Option<LuminaryUser>> {
        let user = sqlx::query_as!(
            LuminaryUser,
            "SELECT [user].* FROM [user] INNER JOIN [session] ON [user].[uuid] = [session].[user] WHERE [session].[token] = ?", token
        )
        .fetch_optional(&self.pool)
        .await
        .wrap_err("Failed to look up session")?;

        return Ok(user);
    }

    /// Creates a new user with the given username and a random reset token, returning the reset token.
    #[wrap_err("Failed to create user")]
    pub async fn create_user(&self, username: &str) -> Result<String> {
        let uuid = Uuid::new_v4().to_string();
        let token = generate_token();

        sqlx::query!(
            "INSERT INTO [user] ([uuid], [username], [reset_token]) VALUES (?, ?, ?)",
            uuid,
            username,
            token
        )
        .execute(&self.pool)
        .await
        .wrap_err("Failed to create user")?;

        return Ok(token);
    }

    /// Sets a user's password to the given value.
    #[wrap_err("Failed to update password")]
    pub async fn set_password(&self, uuid: &str, password: String) -> Result<()> {
        let hashed_password = tokio::task::spawn_blocking(move || password_auth::generate_hash(password))
            .await
            .wrap_err("Failed to spawn hashing task")?;

        sqlx::query!(
            "UPDATE [user] SET [password] = ?, [reset_token] = NULL WHERE [uuid] = ?",
            hashed_password,
            uuid
        )
        .execute(&self.pool)
        .await
        .wrap_err("Failed to set user password")?;

        return Ok(());
    }

    /// Finds a user from their reset token, or [None] if the token is invalid.
    #[wrap_err("Failed to get user from reset token")]
    pub async fn get_user_from_reset_token(&self, token: &str) -> Result<Option<LuminaryUser>> {
        let user = sqlx::query_as!(
            LuminaryUser,
            "SELECT * FROM [user] WHERE [reset_token] = ?",
            token
        )
        .fetch_optional(&self.pool)
        .await
        .wrap_err("Failed to look up user from reset token")?;

        return Ok(user);
    }

    /// Deletes a user from the database, along with all their sessions.
    #[wrap_err("Failed to delete user")]
    pub async fn delete_user(&self, uuid: &str) -> Result<()> {
        sqlx::query!("DELETE FROM [user] WHERE [uuid] = ?", uuid)
            .execute(&self.pool)
            .await
            .wrap_err("Failed to delete user")?;

        return Ok(());
    }

    /// Fetches a list of all users.
    #[wrap_err("Failed to get users")]
    pub async fn get_users(&self) -> Result<Vec<LuminaryUser>> {
        let users = sqlx::query_as!(LuminaryUser, "SELECT * FROM [user]")
            .fetch_all(&self.pool)
            .await
            .wrap_err("Failed to query users")?;

        return Ok(users);
    }
}

/// Salvo middleware for validating authentication.
///
/// On success the authenticated [`LuminaryUser`] is inserted into the depot under `"user"`.
///
/// # Examples
/// ```
/// use salvo::{Router, oapi::endpoint};
/// use crate::api::auth::protected;
///
/// let router = Router::new().hoop(protected).get(protected_handler);
///
/// #[endpoint]
/// async fn protected_handler() -> &'static str {
///    "You are authenticated!"
/// }
/// ```
#[handler]
pub async fn protected(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    let Some(token) = extract_token(req) else {
        res.status_code(StatusCode::UNAUTHORIZED);
        return;
    };

    // Obtain auth backend from the depot
    let auth = obtain!(depot, LuminaryAuthentication);

    match auth
        .get_user_by_token(token)
        .await
        .wrap_err("Failed verify token")
    {
        Ok(Some(user)) => {
            depot.insert("user", user);
            ctrl.call_next(req, depot, res).await;
        }
        Ok(None) => {
            res.status_code(StatusCode::UNAUTHORIZED);
        }
        Err(error) => {
            error!("{}", eyre_fmt!(error));
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

pub fn extract_token(req: &Request) -> Option<&str> {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// Stores the credentials a user would use to log in.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LuminaryUserCredentials {
    pub username: String,
    pub password: String,
}

/// Represents a user in Luminary Node.
#[derive(Clone, Serialize, ToSchema, FromRow)]
pub struct LuminaryUser {
    uuid: String,
    username: String,
    #[serde(skip_serializing)]
    password: Option<String>,
    #[serde(skip_serializing)]
    reset_token: Option<String>,
}

impl Debug for LuminaryUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuminaryUser")
            .field("id", &self.uuid)
            .field("username", &self.username)
            .field("password", &"***")
            .field(
                "reset_token",
                if self.reset_token.is_none() {
                    &"None"
                } else {
                    &"Some(***)"
                },
            )
            .finish()
    }
}

/// Generates a secure random token for authentication purposes.
fn generate_token() -> String {
    let mut token_bytes = [0u8; 32];
    ChaCha12Rng::from_entropy().fill_bytes(&mut token_bytes);
    return hex::encode(token_bytes);
}
