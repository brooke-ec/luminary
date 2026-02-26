use eyre::Result;
use log::error;
use salvo::{
    Depot, Request, Router, Writer,
    http::StatusError,
    oapi::{endpoint, extract::JsonBody},
};

use crate::auth::{LuminaryAuthentication, LuminaryUserCredentials, extract_token};

pub fn router() -> Router {
    return Router::with_path("/auth")
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("logout").post(logout));
}

#[endpoint]
async fn login(depot: &mut Depot, body: JsonBody<LuminaryUserCredentials>) -> Result<String, StatusError> {
    let auth = depot
        .obtain::<LuminaryAuthentication>()
        .expect("Depot partially populated");

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

#[endpoint]
async fn logout(req: &mut Request, depot: &mut Depot) -> Result<(), StatusError> {
    let auth = depot
        .obtain::<LuminaryAuthentication>()
        .expect("Depot partially populated");

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
