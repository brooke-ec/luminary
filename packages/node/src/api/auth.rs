//! A module containing all api endpoints related to authentication.

use eyre::Result;
use salvo::{
    Depot, Request, Router, Writer,
    http::StatusError,
    oapi::{ToSchema, endpoint, extract::JsonBody},
    writing::Json,
};
use serde::Serialize;

use crate::{
    auth::{LuminaryAuthentication, LuminaryUserCredentials, extract_token},
    obtain,
    util::IntoStatusError,
};

/// Returns a router containing all authentication-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/auth")
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("logout").post(logout));
}

/// Reads username and password from the request body, and returns an authentication token if the credentials are valid.
#[endpoint]
async fn login(
    depot: &mut Depot,
    body: JsonBody<LuminaryUserCredentials>,
) -> Result<Json<LoginResponse>, StatusError> {
    let auth = obtain!(depot, LuminaryAuthentication);

    return match auth.login(body.into_inner()).await.into_500()? {
        Some(token) => Ok(Json(LoginResponse { token })),
        None => Err(StatusError::forbidden().brief("Invalid username or password")),
    };
}

#[derive(Debug, Clone, Serialize, ToSchema)]
struct LoginResponse {
    token: String,
}

/// Logs out the current user, invalidating their authentication token.
#[endpoint]
async fn logout(req: &mut Request, depot: &mut Depot) -> Result<(), StatusError> {
    let auth = obtain!(depot, LuminaryAuthentication);

    let Some(token) = extract_token(req) else {
        return Err(StatusError::unauthorized().brief("Missing or invalid authorization token"));
    };

    return auth.logout(token).await.into_500();
}
