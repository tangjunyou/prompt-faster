mod example_impl;

use std::sync::Arc;

use crate::core::traits::TeacherModel;

pub use example_impl::ExampleTeacherModel;

/// TeacherModel 实现选择（用于扩展点示例与本地验证；非用户级 `teacher_llm.model_id`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeacherModelType {
    Example,
}

/// TeacherModel 工厂：新增实现仅需在此处注册（单一入口点）。
pub fn create_teacher_model(teacher_model_type: TeacherModelType) -> Arc<dyn TeacherModel> {
    match teacher_model_type {
        TeacherModelType::Example => Arc::new(ExampleTeacherModel::new_default()),
    }
}

