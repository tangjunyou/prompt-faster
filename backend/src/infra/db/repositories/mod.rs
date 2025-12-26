//! Repository 模块
//! 数据库访问的唯一入口

pub mod credential_repo;
pub mod migration_repo;
pub mod teacher_settings_repo;
pub mod user_repo;
pub mod workspace_repo;

pub use credential_repo::{
    CredentialRecord, CredentialRepo, CredentialRepoError, CredentialType, UpsertCredentialInput,
};
pub use migration_repo::{MigrationRepo, MigrationRepoError, MigrationResult};
pub use teacher_settings_repo::{
    TeacherSettingsRecord, TeacherSettingsRepo, TeacherSettingsRepoError,
    UpsertTeacherSettingsInput,
};
pub use user_repo::{UserRepo, UserRepoError};
pub use workspace_repo::{WorkspaceRepo, WorkspaceRepoError};
