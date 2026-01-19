-- 添加回滚支持字段
-- 注意：branch_id / parent_id 已在 7.1 schema 中存在
ALTER TABLE checkpoints ADD COLUMN archived_at INTEGER;
ALTER TABLE checkpoints ADD COLUMN archive_reason VARCHAR(255);
ALTER TABLE checkpoints ADD COLUMN pass_rate_summary TEXT;

-- 索引优化
CREATE INDEX idx_checkpoints_branch_id ON checkpoints(branch_id);
CREATE INDEX idx_checkpoints_archived_at ON checkpoints(archived_at);
CREATE INDEX idx_checkpoints_parent ON checkpoints(parent_id);
