-- 优化任务（OptimizationTask）表与关联表
-- Story 3.1: 优化任务创建与基本配置

CREATE TABLE IF NOT EXISTS optimization_tasks (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    goal TEXT NOT NULL,
    execution_target_type TEXT NOT NULL, -- dify | generic
    task_mode TEXT NOT NULL, -- fixed | creative
    status TEXT NOT NULL, -- draft（创建态默认）
    config_json TEXT,
    created_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    updated_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_optimization_tasks_workspace_id ON optimization_tasks(workspace_id);

-- 任务-测试集关联表（支持 1..N）
CREATE TABLE IF NOT EXISTS optimization_task_test_sets (
    optimization_task_id TEXT NOT NULL,
    test_set_id TEXT NOT NULL,
    created_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    FOREIGN KEY (optimization_task_id) REFERENCES optimization_tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (test_set_id) REFERENCES test_sets(id) ON DELETE CASCADE,
    UNIQUE (optimization_task_id, test_set_id)
);

CREATE INDEX IF NOT EXISTS idx_optimization_task_test_sets_optimization_task_id
    ON optimization_task_test_sets(optimization_task_id);

