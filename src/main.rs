mod telemetry;

use askama::Template;
use askama_axum::Response;
use axum::{
    body::Bytes,
    extract::MatchedPath,
    http::{HeaderMap, Request},
    routing::get,
    Router,
};
use std::{net::SocketAddr, time::Duration};
use tower_http::{classify::ServerErrorsFailureClass, services::ServeDir, trace::TraceLayer};
use tower_livereload::LiveReloadLayer;

use tracing::{info_span, Span};

use crate::telemetry::{get_subscriber, init_subscriber};

mod filters {
    /// custom filter which can be used in templates as {{ x|reverse }}
    pub fn reverse<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
        let s = s.to_string();
        Ok(s.chars().rev().collect::<String>())
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "clicked.html")]
struct ClickedTemplate {}

async fn index() -> Index<'static> {
    Index { name: "Otherwise" }
}

async fn clicked() -> ClickedTemplate {
    ClickedTemplate {}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = get_subscriber("axumtmx".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let app = Router::new()
        .route("/", get(index))
        .route("/clicked", get(clicked))
        .nest_service("/dist", ServeDir::new("dist"))
        .layer(LiveReloadLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_request(|_request: &Request<_>, _span: &Span| {
                    // You can use `_span.record("some_other_field", value)` in one of these
                    // closures to attach a value to the initially empty field in the info_span
                    // created above.
                })
                .on_response(|_response: &Response, _latency: Duration, _span: &Span| {
                    // ...
                })
                .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {
                    // ...
                })
                .on_eos(
                    |_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {
                        // ...
                    },
                )
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        // ...
                    },
                ),
        );
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
