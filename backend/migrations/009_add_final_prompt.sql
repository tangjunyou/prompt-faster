-- 迁移 009: 添加终止任务相关字段
-- Story: 6-5-iteration-control-add-rounds-manual-terminate
-- 功能: 支持用户手动终止任务并保存选定的 Prompt

-- 添加最终选定 Prompt 字段
ALTER TABLE optimization_tasks ADD COLUMN final_prompt TEXT;

-- 添加终止时间戳（Unix 毫秒）
ALTER TABLE optimization_tasks ADD COLUMN terminated_at INTEGER;

-- 添加选定的迭代 ID（关联 iterations 表）
ALTER TABLE optimization_tasks ADD COLUMN selected_iteration_id TEXT;
