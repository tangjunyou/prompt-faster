-- API 凭证与老师模型参数表
-- Story 1.5: 凭证持久化与老师模型参数配置

-- API 凭证表
-- 存储 Dify 和通用大模型的加密凭证
CREATE TABLE IF NOT EXISTS api_credentials (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL DEFAULT 'default_user',
    credential_type TEXT NOT NULL, -- 'dify' | 'generic_llm'
    provider TEXT, -- 仅 generic_llm 使用: 'siliconflow' | 'modelscope'
    base_url TEXT NOT NULL,
    encrypted_api_key BLOB NOT NULL, -- AES-GCM 加密后的 API Key
    nonce BLOB NOT NULL, -- 12 字节随机数 (AES-GCM 标准)
    salt BLOB NOT NULL, -- Argon2 派生密钥用的盐值
    created_at INTEGER NOT NULL, -- Unix 毫秒时间戳 (AR3)
    updated_at INTEGER NOT NULL, -- Unix 毫秒时间戳 (AR3)
    UNIQUE(user_id, credential_type) -- 每个用户每种凭证类型只能有一条记录
);

-- 老师模型参数表
-- 存储老师模型的调用参数配置
CREATE TABLE IF NOT EXISTS teacher_model_settings (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL DEFAULT 'default_user',
    temperature REAL NOT NULL DEFAULT 0.7, -- 0.0 ~ 2.0
    top_p REAL NOT NULL DEFAULT 0.9, -- 0.0 ~ 1.0
    max_tokens INTEGER NOT NULL DEFAULT 2048, -- 1 ~ 8192
    created_at INTEGER NOT NULL, -- Unix 毫秒时间戳 (AR3)
    updated_at INTEGER NOT NULL, -- Unix 毫秒时间戳 (AR3)
    UNIQUE(user_id) -- 每个用户只能有一条设置记录
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_api_credentials_user_id ON api_credentials(user_id);
CREATE INDEX IF NOT EXISTS idx_api_credentials_type ON api_credentials(user_id, credential_type);
CREATE INDEX IF NOT EXISTS idx_teacher_model_settings_user_id ON teacher_model_settings(user_id);
