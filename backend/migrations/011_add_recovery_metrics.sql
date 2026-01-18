-- Recovery metrics for checkpoint recovery rate tracking
CREATE TABLE IF NOT EXISTS recovery_metrics (
    task_id TEXT PRIMARY KEY,
    success_count INTEGER NOT NULL DEFAULT 0,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(task_id) REFERENCES optimization_tasks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recovery_metrics_task_id
    ON recovery_metrics(task_id);
