mod api;
mod config;
use crate::{api::log_visitor, config::Config};
use axum::{routing::get, Extension, Router};
use clap::Parser;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("SPVC_LOG").unwrap_or_else(|_| "spvc=info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(Config::parse());

    let db_url = format!("sqlite://{}", config.db_path);

    let db = if !Sqlite::database_exists(&db_url).await.unwrap_or_default() {
        Sqlite::create_database(&db_url)
            .await
            .expect("creating SQLite database");
        let pool = SqlitePool::connect(&db_url)
            .await
            .expect("connecting to newly created database");
        let schema = include_str!("../schema.sql");
        sqlx::query(schema)
            .execute(&pool)
            .await
            .expect("loading new database schema");
        pool
    } else {
        SqlitePool::connect(&db_url)
            .await
            .expect("connecting to already existing database")
    };

    let app = Router::new()
        .route("/api/log_visitor", get(log_visitor))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(db.clone()))
        .layer(Extension(config.clone()))
        .layer(CookieManagerLayer::new());

    tracing::info!("listening on {}", config.address);

    if let Err(e) = axum::Server::bind(&config.address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!("server failed: {}", e);
    }

    db.close().await;
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("installing Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("installing signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
