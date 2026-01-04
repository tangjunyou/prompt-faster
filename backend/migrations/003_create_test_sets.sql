-- 测试集（TestSet）表
-- Story 2.1: 测试集数据模型与基础 CRUD

CREATE TABLE IF NOT EXISTS test_sets (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    cases_json TEXT NOT NULL, -- JSON: Vec<TestCase>
    created_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    updated_at INTEGER NOT NULL, -- Unix 毫秒时间戳
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_test_sets_workspace_id ON test_sets(workspace_id);
CREATE INDEX IF NOT EXISTS idx_test_sets_workspace_created_at ON test_sets(workspace_id, created_at);

