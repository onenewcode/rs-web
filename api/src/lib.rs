mod flash;
mod posts;
mod response;

use axum::{
    Router,
    http::StatusCode,
    routing::{get, get_service},
};
use migration::MigratorTrait;
use opentelemetry::trace::TracerProvider;
use service::sea_orm::Database;
use std::env;
use std::sync::OnceLock;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use tracing::*;
use tracing_subscriber::{
    Layer, filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt,
};

use uitls::dotenv;

// 使用 OnceLock 来安全地存储日志 guard
static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();
fn test_span() {
    tracing::span!(Level::ERROR, "start");
}
fn init_log() {
    // 从环境变量读取日志配置
    let enable_console_log = env::var("ENABLE_CONSOLE_LOG")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);
    let enable_file_log = env::var("ENABLE_FILE_LOG")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    let enable_opentelemetry_log = env::var("ENABLE_OPENTELEMETRY")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    let mut layers: Vec<Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync>> = Vec::new();

    // 添加控制台日志层
    if enable_console_log {
        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(true)
            .with_filter(LevelFilter::INFO);
        layers.push(Box::new(stdout_layer));
    }

    // 添加文件日志层
    if enable_file_log {
        let file_appender = tracing_appender::rolling::daily("logs", "myapp.log");
        let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

        // 使用 OnceLock 安全地存储 guard
        match LOG_GUARD.set(guard) {
            Ok(_) => {}
            Err(e) => panic!("Failed to set LOG_GUARD: {:?}", e),
        }

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking_file)
            .with_ansi(false)
            .with_filter(LevelFilter::DEBUG);

        layers.push(Box::new(file_layer));
    }

    // 添加 OpenTelemetry 追踪层
    if enable_opentelemetry_log {
        // 创建 OpenTelemetry 追踪层,该追踪器默认导出到 stdout
        let exporter = opentelemetry_stdout::SpanExporter::default();

        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();

        let tracer = provider.tracer("rs-web-tracer");

        // 创建一个tracing层，使用配置好的tracer
        let telemetry_layer = tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_filter(LevelFilter::INFO);

        layers.push(Box::new(telemetry_layer));
    }
    // 初始化所有日志层
    tracing_subscriber::registry().with(layers).init();
    if enable_opentelemetry_log {
        // 测试 OpenTelemetry 追踪层
        test_span();
    }
}

pub async fn start() -> anyhow::Result<()> {
    match dotenv() {
        Ok(_) => info!("Successfully loaded .env file"),
        Err(e) => {
            error!("Failed to load .env file: {}", e);
            std::process::exit(1);
        }
    }
    init_log();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");

    // Run migrations and seeders
    match migration::Migrator::up(&conn, None).await {
        Ok(_) => info!("Migrations completed successfully"),
        Err(e) => warn!("Migration warning: {}", e),
    }

    match seeder::Migrator::up(&conn, None).await {
        Ok(_) => info!("Seeders completed successfully"),
        Err(e) => warn!("Seeding warning: {}", e),
    }

    let app = Router::new()
        .route("/posts", get(posts::list).post(posts::create))
        .route(
            "/posts/{id}",
            get(posts::show).post(posts::update).delete(posts::delete),
        )
        // 静态文件服务
        .nest_service(
            "/static",
            get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )))
            .handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        // 添加中间件和状态
        .layer(CookieManagerLayer::new())
        .with_state(conn);

    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
