-- 测试集模板支持：在 test_sets 表新增 is_template 字段
-- Story 2.3: 测试集模板保存与复用

ALTER TABLE test_sets
ADD COLUMN is_template INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_test_sets_workspace_is_template_created_at
ON test_sets(workspace_id, is_template, created_at);

