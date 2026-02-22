use std::sync::Arc;

use anyhow::Context;
use axum::routing::{get, post};
use tokio::net;

use crate::domain::crowdsrc::ports::CrowdSrcService;
use crate::inbound::http::handlers::api_home::api_home;
use crate::inbound::http::handlers::create_user::create_user;

mod handlers;
mod responses;

pub struct HttpServerConfig<'a> {
    pub port: &'a str,
}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
struct AppState<CS: CrowdSrcService> {
    crwdsrc_service: Arc<CS>,
}

impl HttpServer {
    pub async fn new(
        crwdsrc_service: impl CrowdSrcService,
        config: HttpServerConfig<'_>,
    ) -> Result<Self, anyhow::Error> {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            },
        );

        let state = AppState {
            crwdsrc_service: Arc::new(crwdsrc_service),
        };

        let router = axum::Router::new()
            .nest("/api", api_routes())
            .layer(trace_layer)
            .with_state(state);
        let listener = net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
            .await
            .with_context(|| format!("failed to listen on {}", config.port))?;

        Ok(Self { router, listener })
    }

    /// Runs the HTTP server.
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::debug!("listening on {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router)
            .await
            .context("received error from running server")?;
        Ok(())
    }

    pub fn local_addr(&self) -> Result<std::net::SocketAddr, std::io::Error> {
        self.listener.local_addr()
    }
}

fn api_routes<CS: CrowdSrcService>() -> axum::Router<AppState<CS>> {
    axum::Router::new()
        .route("/", get(api_home))
        .route("/users", post(create_user::<CS>))
}
