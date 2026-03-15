//! Handles authentication for the Luminary Node, including bearer token management and user authentication.

use std::fmt::Debug;

use eyre::{Context, ContextCompat, Result};
use log::error;
use password_auth::verify_password;
use rand_chacha::{
    ChaCha12Rng,
    rand_core::{RngCore, SeedableRng},
};
use salvo::{oapi::extract::JsonBody, prelude::*};
use serde::Deserialize;
use sqlx::{SqlitePool, prelude::FromRow};

use crate::{api::response::LuminaryResponse, obtain};

/// Returns a router containing all authentication-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/auth")
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("logout").post(logout));
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

    /// Authenticates a user with the given credentials and returns a bearer token on success.
    pub async fn login(&self, credentials: LuminaryUserCredentials) -> Result<Option<String>> {
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
            user.filter(|user| verify_password(&credentials.password, &user.password).is_ok())
        })
        .await
        .wrap_err("Password verification task failed")?;

        // Terminate early if the user doesn't exist or the password is wrong
        let user = match user {
            None => return Ok(None),
            Some(u) => u,
        };

        // Generate a secure bearer token using ChaCha12
        let mut token_bytes = [0u8; 32];
        ChaCha12Rng::from_entropy().fill_bytes(&mut token_bytes);
        let token = hex::encode(token_bytes);

        // Store the token in the database, associated with the user
        sqlx::query!(
            "INSERT INTO [session] ([token], [user_agent], [user]) VALUES (?, ?, ?)",
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
}

/// Salvo middleware for validating authentication.
///
/// On success the authenticated [`LuminaryUser`] is inserted into the depot under `"user"`.
///
/// # Examples
/// ```
/// use salvo::{Router, oapi::endpoint};
/// use crate::auth::protected;
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

    match auth.get_user_by_token(token).await {
        Ok(Some(user)) => {
            depot.insert("user", user);
            ctrl.call_next(req, depot, res).await;
        }
        Ok(None) => {
            res.status_code(StatusCode::UNAUTHORIZED);
        }
        Err(error) => {
            error!("Failed verify token: {error:#}");
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
#[derive(Clone, FromRow)]
pub struct LuminaryUser {
    uuid: String,
    username: String,
    password: String,
}

impl Debug for LuminaryUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuminaryUser")
            .field("id", &self.uuid)
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
}
