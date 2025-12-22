//! Prompt Faster - AI Prompt 自动迭代优化系统
//! 主入口点

use axum::{Router, middleware};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use prompt_faster::api::middleware::correlation_id::correlation_id_middleware;
use prompt_faster::api::routes::health;
use prompt_faster::api::state::AppState;
use prompt_faster::infra::db::pool::create_pool;
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

    // 构建应用状态
    let state = AppState { db };

    // 构建路由
    let app = Router::<AppState>::new()
        .nest("/api/v1", health::router::<AppState>())
        .with_state(state)
        .layer(middleware::from_fn(correlation_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // 启动服务器
    let addr: SocketAddr = config.server_addr().parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("✅ Prompt Faster 已启动: http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
