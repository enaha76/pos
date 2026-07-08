mod error;
mod handlers;
mod models;

use axum::{
    extract::{Request, State},
    http::Method,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use error::AppError;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Process-wide change feed. Subscribers (WebSocket clients) get a short string
/// ("checks" or "config") whenever something is mutated.
static EVENTS: OnceLock<broadcast::Sender<String>> = OnceLock::new();

pub fn events() -> &'static broadcast::Sender<String> {
    EVENTS.get_or_init(|| broadcast::channel::<String>(64).0)
}

/// RBAC gate: admin-only routes require an X-User-Id header for a user whose
/// role is 'admin'. Identity-header based (fine on a trusted LAN); a production
/// system would use signed sessions/tokens.
async fn require_admin(
    State(pool): State<PgPool>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(AppError::Unauthorized)?;

    let role: Option<String> =
        sqlx::query_scalar("select role from users where user_id = $1 and active = true")
            .bind(&user_id)
            .fetch_optional(&pool)
            .await?;

    match role.as_deref() {
        Some("admin") => Ok(next.run(req).await),
        _ => Err(AppError::Forbidden),
    }
}

/// After any successful POST, tell connected clients what changed so they refetch.
async fn broadcast_changes(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let res = next.run(req).await;
    if method == Method::POST && res.status().is_success() {
        let is_config = ["/api/zones", "/api/products", "/api/categories", "/api/settings", "/api/assignments", "/api/servers", "/api/shifts"]
            .iter()
            .any(|p| path.starts_with(p));
        let _ = events().send(if is_config { "config" } else { "checks" }.to_string());
    }
    res
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());

    // The db container may still be warming up — retry the initial connection.
    let pool = connect_with_retry(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    hash_user_pins(&pool).await?;

    // Dev CORS: allow the Vite frontend (any origin) to call the API.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Admin-only: reports, user management, and all owner-managed config writes.
    let admin = Router::new()
        .route("/api/reports/summary", get(handlers::reports_summary))
        .route("/api/reports/daily", get(handlers::reports_daily))
        .route("/api/users", get(handlers::list_users).post(handlers::create_user))
        .route("/api/users/update", post(handlers::update_user))
        .route("/api/users/set-pin", post(handlers::set_user_pin))
        .route("/api/settings", post(handlers::update_settings))
        .route("/api/zones", post(handlers::upsert_zone))
        .route("/api/categories", post(handlers::upsert_category))
        .route("/api/products", post(handlers::upsert_product))
        .route("/api/servers", post(handlers::upsert_server))
        .route("/api/shifts", post(handlers::upsert_shift))
        .route("/api/assignments/toggle", post(handlers::toggle_assignment))
        .route_layer(middleware::from_fn_with_state(pool.clone(), require_admin));

    // Public / any signed-in user (cashier or admin): login + the cashier flow.
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/login", post(handlers::login))
        .route("/ws", get(handlers::ws_handler))
        .route("/api/config", get(handlers::get_config))
        .route("/api/checks/open", get(handlers::list_open))
        .route("/api/checks", post(handlers::create_check))
        .route("/api/checks/add-item", post(handlers::add_item))
        .route("/api/checks/send", post(handlers::send))
        .route("/api/checks/pay", post(handlers::pay))
        .route("/api/checks/close-unpaid", post(handlers::close_unpaid))
        .route("/api/checks/set-server", post(handlers::set_check_server))
        .route("/api/checks/set-table", post(handlers::set_check_table))
        .route("/api/items/void", post(handlers::void_item))
        .route("/api/items/comp", post(handlers::comp_item))
        .route("/api/items/set-qty", post(handlers::set_item_qty))
        .merge(admin)
        .layer(middleware::from_fn(broadcast_changes))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

/// One-time migration of any plaintext user PIN (e.g. the demo seed) to bcrypt.
/// Idempotent — bcrypt hashes start with "$2" and are skipped.
async fn hash_user_pins(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("select user_id, pin_hash from users").fetch_all(pool).await?;
    for (user_id, value) in rows {
        if !value.starts_with("$2") {
            let hash = bcrypt::hash(&value, bcrypt::DEFAULT_COST)?;
            sqlx::query("update users set pin_hash = $2 where user_id = $1")
                .bind(&user_id)
                .bind(&hash)
                .execute(pool)
                .await?;
            tracing::info!("hashed plaintext PIN for {user_id}");
        }
    }
    Ok(())
}

async fn connect_with_retry(database_url: &str) -> anyhow::Result<sqlx::PgPool> {
    let mut attempt = 0;
    loop {
        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                attempt += 1;
                if attempt > 15 {
                    return Err(e.into());
                }
                tracing::warn!("database not ready ({e}); retrying in 2s ({attempt}/15)");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
