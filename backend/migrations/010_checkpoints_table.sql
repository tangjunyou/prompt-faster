-- Checkpoint 表（checkpoints）
-- Story 7.1: Checkpoint 自动保存

CREATE TABLE IF NOT EXISTS checkpoints (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    iteration INTEGER NOT NULL,
    state TEXT NOT NULL, -- IterationState JSON
    run_control_state TEXT NOT NULL, -- RunControlState 枚举值
    prompt TEXT NOT NULL,
    rule_system TEXT NOT NULL, -- RuleSystem JSON
    artifacts TEXT, -- IterationArtifacts JSON
    user_guidance TEXT, -- UserGuidance JSON
    branch_id TEXT NOT NULL,
    parent_id TEXT,
    lineage_type TEXT NOT NULL,
    branch_description TEXT,
    checksum TEXT NOT NULL,
    created_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    FOREIGN KEY (task_id) REFERENCES optimization_tasks(id) ON DELETE CASCADE
);

-- 索引：按任务 ID 查询
CREATE INDEX IF NOT EXISTS idx_checkpoints_task_id ON checkpoints(task_id);

-- 索引：按时间倒序
CREATE INDEX IF NOT EXISTS idx_checkpoints_created_at ON checkpoints(created_at DESC);

-- 复合索引：任务 + 时间倒序（列表查询主路径）
CREATE INDEX IF NOT EXISTS idx_checkpoints_task_created_at ON checkpoints(task_id, created_at DESC);
