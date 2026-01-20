-- 迁移 015: 优化任务关联老师模型 Prompt 版本
-- Story 8.3: 元优化基础

ALTER TABLE optimization_tasks ADD COLUMN teacher_prompt_version_id TEXT REFERENCES teacher_prompts(id);

CREATE INDEX IF NOT EXISTS idx_optimization_tasks_teacher_prompt_version_id
    ON optimization_tasks(teacher_prompt_version_id);
