-- Add Dify config JSON column to test_sets
-- Story 2.4: Dify 变量解析与 Prompt 变量指定

ALTER TABLE test_sets
ADD COLUMN dify_config_json TEXT;

