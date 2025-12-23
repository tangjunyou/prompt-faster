//! Repository 模块
//! 数据库访问的唯一入口

pub mod credential_repo;
pub mod teacher_settings_repo;

pub use credential_repo::{
    CredentialRecord, CredentialRepo, CredentialRepoError, CredentialType, UpsertCredentialInput,
};
pub use teacher_settings_repo::{
    TeacherSettingsRecord, TeacherSettingsRepo, TeacherSettingsRepoError,
    UpsertTeacherSettingsInput,
};
