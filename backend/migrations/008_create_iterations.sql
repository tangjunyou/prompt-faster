-- 迭代历史表（iterations）
-- Story 6.4: 历史迭代产物查看

CREATE TABLE IF NOT EXISTS iterations (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    round INTEGER NOT NULL,
    started_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    completed_at INTEGER, -- Unix 毫秒时间戳（进行中为 NULL）
    status TEXT NOT NULL, -- running/completed/failed/terminated
    artifacts TEXT, -- JSON: IterationArtifacts
    evaluation_results TEXT, -- JSON: EvaluationResultSummary[]
    reflection_summary TEXT,
    pass_rate REAL NOT NULL DEFAULT 0.0,
    total_cases INTEGER NOT NULL DEFAULT 0,
    passed_cases INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    FOREIGN KEY (task_id) REFERENCES optimization_tasks(id) ON DELETE CASCADE
);

-- 索引：按任务 ID 查询
CREATE INDEX IF NOT EXISTS idx_iterations_task_id ON iterations(task_id);

-- 索引：按任务 ID + 轮次倒序（历史列表查询）
CREATE INDEX IF NOT EXISTS idx_iterations_task_id_round_desc ON iterations(task_id, round DESC);
