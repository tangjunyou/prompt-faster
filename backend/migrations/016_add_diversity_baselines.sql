-- 多样性基准线表
-- Story 8.6: 创意任务多样性检测（Growth）

CREATE TABLE IF NOT EXISTS diversity_baselines (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL UNIQUE,
    metrics_json TEXT NOT NULL,
    recorded_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    iteration INTEGER NOT NULL,
    FOREIGN KEY (task_id) REFERENCES optimization_tasks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_diversity_baselines_task_id ON diversity_baselines(task_id);
