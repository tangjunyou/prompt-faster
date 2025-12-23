//! Prompt Faster - AI Prompt 自动迭代优化系统
//! 主入口点

use axum::http::{HeaderName, HeaderValue, Method, header};
use axum::{Router, middleware};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use prompt_faster::api::middleware::correlation_id::{
    CORRELATION_ID_HEADER, correlation_id_middleware,
};
use prompt_faster::api::routes::{auth, health};
use prompt_faster::api::state::AppState;
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::external::api_key_manager::ApiKeyManager;
use prompt_faster::infra::external::http_client::create_http_client;
use prompt_faster::shared::config::AppConfig;
use prompt_faster::shared::tracing_setup::init_tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载配置
    let config = AppConfig::from_env()?;

    // 初始化日志
    init_tracing(&config.log_level);

    info!("Prompt Faster 启动中...");
    info!("服务地址: {}", config.server_addr());

    // 创建数据目录
    std::fs::create_dir_all("data")?;

    // 初始化数据库连接池（包含 WAL/FULL synchronous 设置）
    let db = create_pool(&config.database_url).await?;

    // 自动运行 migrations（确保 schema 就绪）
    sqlx::migrate!().run(&db).await?;

    // 初始化 HTTP 客户端（复用连接池，提高性能）
    let http_client = create_http_client()?;
    info!("HTTP 客户端初始化成功");

    // 初始化 API Key 管理器
    // TODO(Story-1.6): 替换为用户登录密码派生，当前使用临时主密码
    //
    // 安全约束 (Story 1.5 Task 2.2):
    // - 非开发模式下必须设置 MASTER_PASSWORD 环境变量
    // - 密钥仅存于内存，不持久化到文件
    let master_password = match std::env::var("MASTER_PASSWORD") {
        Ok(pwd) if !pwd.is_empty() => {
            info!("使用环境变量中的主密码");
            pwd
        }
        _ => {
            #[cfg(debug_assertions)]
            {
                tracing::warn!("⚠️ 使用默认开发密码，请勿在生产环境使用！");
                "default_dev_password_change_me".to_string()
            }
            #[cfg(not(debug_assertions))]
            {
                panic!("❌ 生产模式必须设置 MASTER_PASSWORD 环境变量！请在 .env 或环境中配置安全的主密码。");
            }
        }
    };
    let api_key_manager = Arc::new(ApiKeyManager::new(master_password));
    info!("API Key 管理器初始化成功");

    // 构建应用状态
    let state = AppState {
        db,
        http_client,
        api_key_manager,
    };

    // 允许的前端 Origin（从配置读取）
    let allowed_origins: Vec<HeaderValue> = config
        .cors_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    // x-correlation-id header 名称
    let correlation_id_header: HeaderName = CORRELATION_ID_HEADER.parse().unwrap();

    // 构建路由
    let app = Router::<AppState>::new()
        .nest("/api/v1", health::router::<AppState>())
        .nest("/api/v1/auth", auth::router())
        .with_state(state)
        .layer(middleware::from_fn(correlation_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    header::ACCEPT,
                    correlation_id_header.clone(),
                ])
                .expose_headers([correlation_id_header]),
        );

    // 启动服务器
    let addr: SocketAddr = config.server_addr().parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("✅ Prompt Faster 已启动: http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
