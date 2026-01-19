-- 历史事件表
-- Story 7.4: 完整迭代历史记录

CREATE TABLE IF NOT EXISTS history_events (
    id VARCHAR(36) PRIMARY KEY,
    task_id VARCHAR(36) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    actor VARCHAR(20) NOT NULL,
    details TEXT,
    iteration INTEGER,
    correlation_id VARCHAR(36),
    created_at INTEGER NOT NULL,
    FOREIGN KEY (task_id) REFERENCES optimization_tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_history_events_task_time ON history_events(task_id, created_at);
CREATE INDEX idx_history_events_created_at ON history_events(created_at);
CREATE INDEX idx_history_events_event_type ON history_events(event_type);
