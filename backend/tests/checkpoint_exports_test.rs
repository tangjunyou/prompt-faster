use prompt_faster::core::iteration_engine::checkpoint::CHECKPOINT_MODULE_REGISTERED;
use prompt_faster::domain::models::Checkpoint;
use prompt_faster::infra::db::repositories::CheckpointRepo;
use sha2::{Digest, Sha256};

#[test]
#[allow(clippy::assertions_on_constants)]
fn sha2_is_available_and_checkpoint_modules_are_registered() {
    assert!(CHECKPOINT_MODULE_REGISTERED);
    let _ = std::mem::size_of::<Checkpoint>();
    let _ = std::mem::size_of::<CheckpointRepo>();
    let mut hasher = Sha256::new();
    hasher.update(b"checkpoint");
    let digest = hasher.finalize();
    assert_eq!(digest.len(), 32);
}
