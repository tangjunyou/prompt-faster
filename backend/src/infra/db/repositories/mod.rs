//! Repository 模块
//! 数据库访问的唯一入口

pub mod credential_repo;
pub mod teacher_settings_repo;

pub use credential_repo::{CredentialRepo, CredentialRecord, CredentialType, UpsertCredentialInput, CredentialRepoError};
pub use teacher_settings_repo::{TeacherSettingsRepo, TeacherSettingsRecord, UpsertTeacherSettingsInput, TeacherSettingsRepoError};
