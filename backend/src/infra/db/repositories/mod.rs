//! Repository 模块
//! 数据库访问的唯一入口

pub mod checkpoint_repo;
pub mod credential_repo;
pub mod diversity_baseline_repo;
pub mod history_event_repo;
pub mod iteration_repo;
pub mod migration_repo;
pub mod optimization_task_repo;
pub mod recovery_metrics_repo;
pub mod teacher_prompt_repo;
pub mod teacher_settings_repo;
pub mod test_set_repo;
pub mod user_repo;
pub mod workspace_repo;

pub use checkpoint_repo::{CheckpointRepo, CheckpointRepoError};
pub use credential_repo::{
    CredentialRecord, CredentialRepo, CredentialRepoError, CredentialType, UpsertCredentialInput,
};
pub use diversity_baseline_repo::{DiversityBaselineRepo, DiversityBaselineRepoError};
pub use history_event_repo::{HistoryEventRepo, HistoryEventRepoError};
pub use iteration_repo::{
    IterationRepo, IterationRepoError, IterationSummaryWithArtifacts,
    IterationSummaryWithArtifactsAndEvaluations,
};
pub use migration_repo::{MigrationRepo, MigrationRepoError, MigrationResult};
pub use optimization_task_repo::{
    CreateOptimizationTaskInput, OptimizationTaskRepo, OptimizationTaskRepoError,
};
pub use recovery_metrics_repo::{RecoveryMetricsRepo, RecoveryMetricsRepoError};
pub use teacher_prompt_repo::{
    CreateTeacherPromptRecordInput, TeacherPromptRecord, TeacherPromptRepo, TeacherPromptRepoError,
    TeacherPromptVersionRecord, TeacherPromptVersionWithStatsRecord,
};
pub use teacher_settings_repo::{
    TeacherSettingsRecord, TeacherSettingsRepo, TeacherSettingsRepoError,
    UpsertTeacherSettingsInput,
};
pub use test_set_repo::{TestSetRepo, TestSetRepoError};
pub use user_repo::{UserRepo, UserRepoError};
pub use workspace_repo::{WorkspaceRepo, WorkspaceRepoError};
