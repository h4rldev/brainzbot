use axum::{
    Router,
    http::{
        Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
};
use axum_server::{Address, Handle};
use oauth::{IoResult, render_404};
use std::{net::SocketAddr, str::FromStr, time::Duration};
use tokio::{signal, time::sleep};
use tower::{ServiceBuilder, service_fn};
use tower_http::{
    CompressionLevel,
    compression::{
        CompressionLayer, Predicate,
        predicate::{NotForContentType, SizeAbove},
    },
    cors::{AllowOrigin, CorsLayer},
    decompression::RequestDecompressionLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{Level, info};
use tracing_subscriber::{
    field::MakeExt,
    fmt::{Subscriber, format::debug_fn},
};

mod oauth;

pub fn slice_to_ip(ip: &str) -> Result<[u8; 4], String> {
    let mut ip_bytes = [0; 4];
    let ip = ip.split('.').collect::<Vec<&str>>();
    if ip.len() != 4 {
        return Err(format!("invalid ip address: {:?}", ip));
    }
    for (i, byte) in ip.iter().enumerate() {
        let byte = byte
            .parse::<u8>()
            .map_err(|_| format!("invalid ip address: {:?}", ip))?;
        ip_bytes[i] = byte;
    }
    Ok(ip_bytes)
}

#[tokio::main]
async fn main() -> IoResult<()> {
    let formatter =
        debug_fn(|writer, field, value| write!(writer, "{field}: {value:?}")).delimited(",");

    let level: Level =
        Level::from_str(&std::env::var("LOG_LEVEL").unwrap_or("info".to_string())).unwrap();

    Subscriber::builder()
        .with_max_level(level)
        .fmt_fields(formatter)
        .with_ansi(true)
        .init();

    let handle = Handle::new();
    tokio::spawn(shutdown_signal(handle.clone()));

    let compression_predicate = SizeAbove::new(256).and(NotForContentType::IMAGES);
    let cors_public = if cfg!(debug_assertions) {
        CorsLayer::new()
            .allow_origin(AllowOrigin::any())
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::HEAD,
            ])
            .allow_headers([ACCEPT, CONTENT_TYPE, AUTHORIZATION])
            .max_age(Duration::from_secs(60 * 60 * 24 * 7))
    } else {
        CorsLayer::new()
            .allow_origin(AllowOrigin::any())
            .allow_methods([Method::GET, Method::HEAD])
            .allow_headers([ACCEPT, CONTENT_TYPE])
            .max_age(Duration::from_secs(60 * 60 * 24))
    };

    let app = Router::new()
        .fallback_service(service_fn(render_404))
        .layer(cors_public)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http().make_span_with(
                        DefaultMakeSpan::new()
                            .level(tracing::Level::INFO)
                            .include_headers(false),
                    ),
                )
                .layer(RequestDecompressionLayer::new())
                .layer(
                    CompressionLayer::new()
                        .no_br()
                        .no_deflate()
                        .gzip(true)
                        .zstd(true)
                        .quality(CompressionLevel::Fastest)
                        .compress_when(compression_predicate),
                ),
        );
    // .with_state(state);

    let ip = slice_to_ip(&std::env::var("IP").unwrap_or_else(|_| "0.0.0.0".to_string()))
        .unwrap_or_else(|e| panic!("Invalid ip: {e}"));
    let addr = SocketAddr::from((ip, 8080));

    info!("Serving HTTP on {addr}");
    axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
}

async fn shutdown_signal<A>(handle: Handle<A>)
where
    A: Address,
{
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    eprintln!("\n");
    tracing::info!("Received CTRL-C shutting down gracefully");
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
    loop {
        sleep(Duration::from_secs(1)).await;
        info!("alive connections: {}", handle.connection_count());
    }
}
