use crate::config::Config;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::{header::CONTENT_TYPE, StatusCode},
    response::IntoResponse,
    Extension,
};
use axum_client_ip::ClientIp;
use cookie::{time::Duration, SameSite};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};

pub async fn log_visitor(
    headers: ExtractedHeaders,
    ClientIp(ip): ClientIp,
    cookies: Cookies,
    Extension(config): Extension<Arc<Config>>,
    Extension(db): Extension<Pool<Sqlite>>,
) -> impl IntoResponse {
    if !config
        .allowed_urls
        .iter()
        .any(|prefix| headers.referer.starts_with(prefix))
    {
        tracing::warn!(
            "unauthorized call from referer: {} with IP: {} and user agent: {}",
            headers.referer,
            ip,
            headers.user_agent
        );

        return (
            StatusCode::UNAUTHORIZED,
            [(CONTENT_TYPE, "text/javascript")],
        );
    }

    let url_id: i64 = sqlx::query(
        "\
INSERT INTO urls (url)
VALUES (?)
ON CONFLICT DO UPDATE SET id = id RETURNING id;
",
    )
    .bind(headers.referer)
    .fetch_one(&db)
    .await
    .expect("getting a single row with one integer")
    .get(0);

    let visitor_id: i64 = match cookies.get("visitor_id") {
        Some(cookie) => sqlx::query(
            "\
INSERT INTO visitors (id)
VALUES (?)
ON CONFLICT DO UPDATE SET id = id RETURNING id;
",
        )
        .bind(cookie.value().parse::<i64>().unwrap_or_else(|e| {
            tracing::error!("cookie parsing failed: {}", e);
            1
        }))
        .fetch_one(&db)
        .await
        .expect("getting a single row with one integer")
        .get(0),
        None => sqlx::query(
            "\
INSERT INTO visitors (first_visit)
VALUES (datetime('now'))
ON CONFLICT DO UPDATE SET id = id RETURNING id;
",
        )
        .fetch_one(&db)
        .await
        .expect("getting a single row with one integer")
        .get(0),
    };

    let mut visitor_cookie = Cookie::new("visitor_id", visitor_id.to_string());
    visitor_cookie.set_same_site(SameSite::None);
    visitor_cookie.set_secure(true);
    visitor_cookie.set_max_age(Duration::days(365));
    cookies.add(visitor_cookie);

    let user_agent_id: i64 = if config.save_user_agent {
        sqlx::query(
            "\
INSERT INTO user_agents (user_agent)
VALUES (?)
ON CONFLICT DO UPDATE SET id = id RETURNING id;
",
        )
        .bind(headers.user_agent)
        .fetch_one(&db)
        .await
        .expect("getting a single row with one integer")
        .get(0)
    } else {
        1
    };

    let ip_id: i64 = if config.save_ip {
        sqlx::query(
            "\
INSERT INTO ips (ip)
VALUES (?)
ON CONFLICT DO UPDATE SET id = id RETURNING id;
",
        )
        .bind(ip.to_string())
        .fetch_one(&db)
        .await
        .expect("getting a single row with one integer")
        .get(0)
    } else {
        1
    };

    if let Err(e) = sqlx::query(
        "\
INSERT INTO visits (url, visitor, user_agent, ip)
VALUES (?, ?, ?, ?);
",
    )
    .bind(url_id)
    .bind(visitor_id)
    .bind(user_agent_id)
    .bind(ip_id)
    .execute(&db)
    .await
    {
        tracing::error!("Failed to add visit: {}", e);
    };

    (StatusCode::OK, [(CONTENT_TYPE, "text/javascript")])
}

pub struct ExtractedHeaders {
    referer: String,
    user_agent: String,
}

#[async_trait]
impl<B> FromRequest<B> for ExtractedHeaders
where
    B: Send,
{
    type Rejection = ();

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let referer = match req.headers().get("referer") {
            Some(value) => match value.to_str() {
                Ok(host) => host.to_owned(),
                Err(e) => {
                    let invalid_msg = "INVALID_REFERER_HEADER";
                    tracing::error!("{}: {}", invalid_msg, e);
                    invalid_msg.to_string()
                }
            },
            None => "MISSING_REFERER_HEADER".to_string(),
        };

        let user_agent = match req.headers().get("user-agent") {
            Some(value) => match value.to_str() {
                Ok(host) => host.to_owned(),
                Err(e) => {
                    let invalid_msg = "INVALID_USER_AGENT_HEADER";
                    tracing::error!("{}: {}", invalid_msg, e);
                    invalid_msg.to_string()
                }
            },
            None => "MISSING_USER_AGENT_HEADER".to_string(),
        };

        Ok(Self {
            referer,
            user_agent,
        })
    }
}
