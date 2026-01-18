//! 后端配置唯一入口
//! 所有模块从此获取配置，不得直接读 env

use std::env;
use std::path::Path;

/// 应用配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 数据库 URL
    pub database_url: String,
    /// 服务器地址
    pub server_host: String,
    /// 服务器端口
    pub server_port: u16,
    /// 日志级别
    pub log_level: String,
    /// 是否为开发模式
    pub is_dev: bool,
    /// CORS 允许的 Origins（逗号分隔）
    pub cors_origins: Vec<String>,
    /// 是否运行在 Docker 容器内（用于本机服务地址处理）
    pub is_docker: bool,
    /// 是否允许 HTTP base_url（仅本地/开发模式建议开启）
    pub allow_http_base_url: bool,
    /// 是否允许 localhost/127.0.0.1 作为 base_url
    pub allow_localhost_base_url: bool,
    /// 是否允许私有网段 IP（10/172.16-31/192.168/169.254、以及 IPv6 ULA/Link-local）
    pub allow_private_network_base_url: bool,
    /// Checkpoint 缓存条数上限（默认 10）
    pub checkpoint_cache_limit: usize,
    /// Checkpoint 缓存告警阈值（默认 10）
    pub checkpoint_memory_alert_threshold: usize,
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let is_dev = env::var("APP_ENV")
            .map(|v| v == "development")
            .unwrap_or(true);

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:data/prompt_faster.db?mode=rwc".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            is_dev,
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://127.0.0.1:5173".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            is_docker: Path::new("/.dockerenv").exists(),
            allow_http_base_url: env::var("ALLOW_HTTP_BASE_URL")
                .ok()
                .as_deref()
                .map(parse_bool_env)
                .unwrap_or(is_dev),
            allow_localhost_base_url: env::var("ALLOW_LOCALHOST_BASE_URL")
                .ok()
                .as_deref()
                .map(parse_bool_env)
                .unwrap_or(is_dev),
            allow_private_network_base_url: env::var("ALLOW_PRIVATE_NETWORK_BASE_URL")
                .ok()
                .as_deref()
                .map(parse_bool_env)
                .unwrap_or(is_dev),
            checkpoint_cache_limit: env::var("CHECKPOINT_CACHE_LIMIT")
                .ok()
                .as_deref()
                .and_then(parse_usize_env)
                .unwrap_or(10),
            checkpoint_memory_alert_threshold: env::var("CHECKPOINT_MEMORY_ALERT_THRESHOLD")
                .ok()
                .as_deref()
                .and_then(parse_usize_env)
                .unwrap_or(10),
        })
    }

    /// 获取完整的服务器地址
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

fn parse_bool_env(v: &str) -> bool {
    matches!(
        v.trim().to_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "on"
    )
}

fn parse_usize_env(v: &str) -> Option<usize> {
    let trimmed = v.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<usize>().ok().filter(|v| *v > 0)
}
