-- Add Generic API custom variables config JSON column to test_sets
-- Story 2.5: 通用 API 自定义变量与固定任务标准答案

ALTER TABLE test_sets
ADD COLUMN generic_config_json TEXT;

