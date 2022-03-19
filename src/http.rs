use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::Extension,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use tracing::{debug, error};

use crate::State;

pub enum ServerError {
    MissingParameter(&'static str),
    WebDriverError,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServerError::MissingParameter(param) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Missing parameter '{}'", param),
            ),
            ServerError::WebDriverError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "WebDriver failed".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub async fn start_server(state: Arc<State>) {
    debug!("Starting HTTP server");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(handlers::root))
        .route("/navigate", get(handlers::navigate))
        .layer(Extension(state));

    // run our app with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("http serve failed");

    error!("server quit unexpectedly");
}

mod handlers {
    use super::*;

    // basic handler that responds with a static string
    pub(crate) async fn root() -> &'static str {
        "Hello, World!"
    }

    pub(crate) async fn navigate(
        Query(params): Query<HashMap<String, String>>,
        Extension(state): Extension<Arc<State>>,
    ) -> Result<String, ServerError> {
        let url = params
            .get("url")
            .ok_or(ServerError::MissingParameter("url"))?;

        let wd = state.driver.webdriver().await.unwrap();
        wd.get(url).await.map_err(|_| ServerError::WebDriverError)?;

        debug!(?url, "navigating to requested url");

        wd.quit().await.map_err(|_| ServerError::WebDriverError)?;

        Ok("ok".to_string())
    }
}
