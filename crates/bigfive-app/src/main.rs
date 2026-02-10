//! Server entry point for the Big Five Personality Test application.

#[cfg(feature = "ssr")]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(feature = "ssr")]
pub const GIT_HASH: &str = env!("GIT_HASH");
#[cfg(feature = "ssr")]
pub const BUILD_TIME: &str = env!("BUILD_TIME");

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum::extract::Request;
    use axum::middleware::{self, Next};
    use axum::response::{IntoResponse, Json};
    use axum::routing::get;
    use axum_governor::GovernorLayer;
    use bigfive_app::app::*;
    use bigfive_app::config::get_config;
    use lazy_limit::{Duration, RuleConfig, init_rate_limiter};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use real::{RealIp, RealIpLayer};
    use serde_json::json;
    use tracing::info;
    use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    dotenvy::dotenv().ok();

    // Initialize tracing with env filter (default: info, configurable via RUST_LOG)
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Starting Big Five Tester v{}-{} (built {})",
        VERSION, GIT_HASH, BUILD_TIME
    );

    // Load and display AI configuration
    match get_config() {
        Ok(config) => {
            info!("AI Configuration loaded:");
            info!("  Available models:");
            for preset in &config.models {
                let default_marker = if preset.default { " (default)" } else { "" };
                info!(
                    "    - {} [{}]: {}{}, source_lang={:?}",
                    preset.display_name,
                    preset.id,
                    preset.model,
                    default_marker,
                    preset.source_lang
                );
            }
            if let Some(ref safeguard) = config.safeguard {
                if safeguard.enabled {
                    info!("  Safeguard: {} (enabled)", safeguard.model);
                } else {
                    info!("  Safeguard: disabled");
                }
            } else {
                info!("  Safeguard: not configured");
            }
        }
        Err(e) => {
            tracing::error!("Failed to load AI config: {}", e);
        }
    }

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Version endpoint
    async fn version_handler() -> Json<serde_json::Value> {
        Json(json!({
            "version": VERSION,
            "git_hash": GIT_HASH,
            "build_time": BUILD_TIME
        }))
    }

    // Rate limiting configuration (per IP):
    // - Default: 60 requests per 10 seconds (generous for normal browsing)
    // - AI analysis: 2 requests per minute (protects expensive API calls)
    init_rate_limiter!(
        default: RuleConfig::new(Duration::seconds(10), 60),
        routes: [
            ("/api/start_analysis", RuleConfig::new(Duration::seconds(60), 2))
        ]
    )
    .await;
    info!("Rate limiting enabled: 60 req/10s default, 2 req/min for AI analysis");

    // Logging middleware that captures real IP
    async fn log_request(req: Request, next: Next) -> impl IntoResponse {
        let ip = req.extensions().get::<RealIp>().map(|r| r.0);
        let method = req.method().clone();
        let uri = req.uri().clone();
        tracing::info!(%method, %uri, ?ip, "request");
        next.run(req).await
    }

    let app = Router::new()
        .route("/api/version", get(version_handler))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(middleware::from_fn(log_request))
        .layer(
            tower::ServiceBuilder::new()
                .layer(RealIpLayer::default())
                .layer(GovernorLayer::default()),
        )
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("Server listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // No client-side main function
}
