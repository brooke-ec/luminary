//! A module containing all api endpoints related to authentication.

use eyre::Result;
use log::error;
use luminary_macros::obtain;
use salvo::{
    Depot, Request, Router, Writer,
    http::StatusError,
    oapi::{endpoint, extract::JsonBody},
};

use crate::auth::{LuminaryAuthentication, LuminaryUserCredentials, extract_token};

/// Returns a router containing all authentication-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/auth")
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("logout").post(logout));
}

/// Reads username and password from the request body, and returns an authentication token if the credentials are valid.
#[endpoint]
async fn login(depot: &mut Depot, body: JsonBody<LuminaryUserCredentials>) -> Result<String, StatusError> {
    let auth = obtain!(depot, LuminaryAuthentication);

    return match auth.login(body.into_inner()).await {
        Ok(Some(token)) => Ok(token),
        Ok(None) => Err(StatusError::forbidden().brief("Invalid username or password")),
        Err(error) => {
            error!("Login failed: {error:#}");
            Err(StatusError::internal_server_error()
                .brief("An error occurred while processing the login request"))
        }
    };
}

/// Logs out the current user, invalidating their authentication token.
#[endpoint]
async fn logout(req: &mut Request, depot: &mut Depot) -> Result<(), StatusError> {
    let auth = obtain!(depot, LuminaryAuthentication);

    let Some(token) = extract_token(req) else {
        return Err(StatusError::unauthorized().brief("Missing or invalid authorization token"));
    };

    return match auth.logout(token).await {
        Ok(()) => Ok(()),
        Err(error) => {
            error!("Logout failed: {error:#}");
            Err(StatusError::internal_server_error().brief("An error occurred while logging out"))
        }
    };
}
