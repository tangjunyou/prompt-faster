-- 迁移 014: 新增老师模型 Prompt 版本表
-- Story 8.3: 元优化基础

CREATE TABLE IF NOT EXISTS teacher_prompts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    version INTEGER NOT NULL,             -- 版本号（用户维度自增）
    content TEXT NOT NULL,                -- Prompt 完整内容
    description TEXT,                     -- 版本变更说明
    is_active INTEGER NOT NULL DEFAULT 0, -- 是否为当前活跃版本（0/1）
    created_at INTEGER NOT NULL,          -- Unix 毫秒时间戳
    updated_at INTEGER NOT NULL,          -- Unix 毫秒时间戳
    UNIQUE(user_id, version)
);

CREATE INDEX IF NOT EXISTS idx_teacher_prompts_user_id ON teacher_prompts(user_id);
CREATE INDEX IF NOT EXISTS idx_teacher_prompts_user_active ON teacher_prompts(user_id, is_active);
