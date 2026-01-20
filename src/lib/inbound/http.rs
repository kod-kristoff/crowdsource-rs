use anyhow::Context;
use axum::routing::get;
use tokio::net;

use crate::inbound::http::handlers::api_home::api_home;

mod handlers;

pub struct HttpServerConfig<'a> {
    pub port: &'a str,
}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new(config: HttpServerConfig<'_>) -> Result<Self, anyhow::Error> {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            },
        );
        let router = axum::Router::new()
            .nest("/api", api_routes())
            .layer(trace_layer);
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
}

fn api_routes() -> axum::Router {
    axum::Router::new().route("/", get(api_home))
}
