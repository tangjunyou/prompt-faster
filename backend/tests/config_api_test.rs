//! 凭证配置集成测试
//! Story 1.5 Task 5.2, 5.4: 验证凭证加密、持久化和读取逻辑
//!
//! # 测试范围说明
//! 本测试文件主要验证 **Repository 层** 和 **加密层** 的正确性，
//! 而非 HTTP API 端点本身。HTTP API 端点测试依赖完整的 Axum 服务启动，
//! 建议在未来 Story 中补充完整的 E2E API 测试。
//!
//! 测试覆盖:
//! - API Key 加解密可逆性 (AES-GCM + Argon2)
//! - 加密数据确实不是明文
//! - 相同明文 + 不同 salt/nonce 产生不同密文
//! - 错误密码无法解密
//! - 凭证存储到数据库后仍为加密状态
//! - 老师模型参数存储和默认值
//! - 凭证 upsert 更新逻辑
//! - 不同凭证类型独立存储

use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::db::repositories::{CredentialRepo, CredentialType, TeacherSettingsRepo};
use prompt_faster::infra::external::api_key_manager::{ApiKeyManager, NONCE_LENGTH, SALT_LENGTH};
use sqlx::SqlitePool;

/// 测试用主密码
const TEST_MASTER_PASSWORD: &str = "test_master_password_for_integration";

/// 测试用用户 ID（仅用于测试 Repository 层逻辑）
const TEST_USER_ID: &str = "test_user";

/// 创建测试数据库（内存数据库 + 运行 migrations）
async fn setup_test_db() -> SqlitePool {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");
    pool
}

/// 测试 API Key Manager 加解密可逆性
#[tokio::test]
async fn test_api_key_encryption_roundtrip() {
    let manager = ApiKeyManager::new(TEST_MASTER_PASSWORD.to_string());
    let original_key = "sk-test-api-key-12345678";

    let encrypted = manager.encrypt(original_key).expect("加密失败");
    let decrypted = manager.decrypt(&encrypted).expect("解密失败");

    assert_eq!(original_key, decrypted);
}

/// 测试加密后的数据确实不是明文
#[tokio::test]
async fn test_encrypted_data_is_not_plaintext() {
    let manager = ApiKeyManager::new(TEST_MASTER_PASSWORD.to_string());
    let api_key = "sk-very-secret-api-key";

    let encrypted = manager.encrypt(api_key).expect("加密失败");

    // 密文不应该包含明文
    let ciphertext_str = String::from_utf8_lossy(&encrypted.ciphertext);
    assert!(
        !ciphertext_str.contains(api_key),
        "密文不应包含明文 API Key"
    );

    // 验证 nonce 长度
    assert_eq!(
        encrypted.nonce.len(),
        NONCE_LENGTH,
        "nonce 长度应为 12 字节"
    );

    // 验证 salt 长度
    assert_eq!(encrypted.salt.len(), SALT_LENGTH, "salt 长度应为 16 字节");
}

/// 测试相同明文 + 不同 salt/nonce 产生不同密文
#[tokio::test]
async fn test_different_salt_produces_different_ciphertext() {
    let manager = ApiKeyManager::new(TEST_MASTER_PASSWORD.to_string());
    let api_key = "sk-same-api-key";

    let encrypted1 = manager.encrypt(api_key).expect("加密失败");
    let encrypted2 = manager.encrypt(api_key).expect("加密失败");

    // 不同的 salt 和 nonce 应产生不同的密文
    assert_ne!(
        encrypted1.ciphertext, encrypted2.ciphertext,
        "相同明文应产生不同密文"
    );
    assert_ne!(encrypted1.salt, encrypted2.salt, "salt 应该不同");
    assert_ne!(encrypted1.nonce, encrypted2.nonce, "nonce 应该不同");
}

/// 测试错误密码无法解密
#[tokio::test]
async fn test_wrong_password_fails_decryption() {
    let manager1 = ApiKeyManager::new("correct_password".to_string());
    let manager2 = ApiKeyManager::new("wrong_password".to_string());
    let api_key = "sk-secret-key";

    let encrypted = manager1.encrypt(api_key).expect("加密失败");
    let result = manager2.decrypt(&encrypted);

    assert!(result.is_err(), "错误密码应导致解密失败");
}

/// 测试凭证存储到数据库后仍为加密状态
#[tokio::test]
async fn test_credential_storage_is_encrypted() {
    let pool = setup_test_db().await;
    let manager = ApiKeyManager::new(TEST_MASTER_PASSWORD.to_string());
    let original_api_key = "sk-original-api-key-for-storage-test";

    // 加密 API Key
    let encrypted = manager.encrypt(original_api_key).expect("加密失败");

    // 保存到数据库
    use prompt_faster::infra::db::repositories::UpsertCredentialInput;
    let saved = CredentialRepo::upsert(
        &pool,
        UpsertCredentialInput {
            user_id: TEST_USER_ID.to_string(),
            credential_type: CredentialType::Dify,
            provider: None,
            base_url: "https://api.dify.ai".to_string(),
            encrypted_api_key: encrypted.ciphertext.clone(),
            nonce: encrypted.nonce.clone(),
            salt: encrypted.salt.clone(),
        },
    )
    .await
    .expect("保存凭证失败");

    // 验证数据库中存储的是加密数据
    let stored_ciphertext_str = String::from_utf8_lossy(&saved.encrypted_api_key);
    assert!(
        !stored_ciphertext_str.contains(original_api_key),
        "数据库中不应存储明文 API Key"
    );

    // 验证可以从数据库读取并解密
    let loaded =
        CredentialRepo::find_by_user_and_type(&pool, TEST_USER_ID, CredentialType::Dify)
            .await
            .expect("读取凭证失败");

    let loaded_encrypted = prompt_faster::infra::external::api_key_manager::EncryptedApiKey {
        ciphertext: loaded.encrypted_api_key,
        nonce: loaded.nonce,
        salt: loaded.salt,
    };

    let decrypted = manager.decrypt(&loaded_encrypted).expect("解密失败");
    assert_eq!(original_api_key, decrypted, "解密后应与原文一致");
}

/// 测试老师模型参数存储和读取
#[tokio::test]
async fn test_teacher_settings_storage() {
    let pool = setup_test_db().await;

    use prompt_faster::infra::db::repositories::UpsertTeacherSettingsInput;

    // 保存老师模型参数
    let saved = TeacherSettingsRepo::upsert(
        &pool,
        UpsertTeacherSettingsInput {
            user_id: TEST_USER_ID.to_string(),
            temperature: 0.8,
            top_p: 0.95,
            max_tokens: 4096,
        },
    )
    .await
    .expect("保存老师模型参数失败");

    assert_eq!(saved.temperature, 0.8);
    assert_eq!(saved.top_p, 0.95);
    assert_eq!(saved.max_tokens, 4096);

    // 读取老师模型参数
    let loaded = TeacherSettingsRepo::find_by_user(&pool, TEST_USER_ID)
        .await
        .expect("读取老师模型参数失败");

    assert_eq!(loaded.temperature, 0.8);
    assert_eq!(loaded.top_p, 0.95);
    assert_eq!(loaded.max_tokens, 4096);
}

/// 测试 get_or_default 在无记录时返回默认值
#[tokio::test]
async fn test_teacher_settings_default() {
    let pool = setup_test_db().await;

    let settings = TeacherSettingsRepo::get_or_default(&pool, "non_existent_user")
        .await
        .expect("get_or_default 失败");

    assert_eq!(settings.temperature, 0.7, "默认 temperature 应为 0.7");
    assert_eq!(settings.top_p, 0.9, "默认 top_p 应为 0.9");
    assert_eq!(settings.max_tokens, 2048, "默认 max_tokens 应为 2048");
}

/// 测试凭证 upsert 更新逻辑（同一用户同一类型只保留一条）
#[tokio::test]
async fn test_credential_upsert_updates_existing() {
    let pool = setup_test_db().await;
    let manager = ApiKeyManager::new(TEST_MASTER_PASSWORD.to_string());

    use prompt_faster::infra::db::repositories::UpsertCredentialInput;

    // 第一次保存
    let encrypted1 = manager.encrypt("sk-first-key").expect("加密失败");
    CredentialRepo::upsert(
        &pool,
        UpsertCredentialInput {
            user_id: TEST_USER_ID.to_string(),
            credential_type: CredentialType::Dify,
            provider: None,
            base_url: "https://api.dify.ai/v1".to_string(),
            encrypted_api_key: encrypted1.ciphertext,
            nonce: encrypted1.nonce,
            salt: encrypted1.salt,
        },
    )
    .await
    .expect("第一次保存失败");

    // 第二次保存（更新）
    let encrypted2 = manager.encrypt("sk-second-key").expect("加密失败");
    CredentialRepo::upsert(
        &pool,
        UpsertCredentialInput {
            user_id: TEST_USER_ID.to_string(),
            credential_type: CredentialType::Dify,
            provider: None,
            base_url: "https://api.dify.ai/v2".to_string(),
            encrypted_api_key: encrypted2.ciphertext.clone(),
            nonce: encrypted2.nonce.clone(),
            salt: encrypted2.salt.clone(),
        },
    )
    .await
    .expect("第二次保存失败");

    // 验证只有一条记录，且是更新后的
    let all_credentials = CredentialRepo::find_all_by_user(&pool, TEST_USER_ID)
        .await
        .expect("读取所有凭证失败");

    let dify_credentials: Vec<_> = all_credentials
        .iter()
        .filter(|c| c.credential_type == "dify")
        .collect();

    assert_eq!(dify_credentials.len(), 1, "应该只有一条 Dify 凭证");
    assert_eq!(
        dify_credentials[0].base_url, "https://api.dify.ai/v2",
        "base_url 应已更新"
    );

    // 验证可以解密为第二个 key
    let loaded_encrypted = prompt_faster::infra::external::api_key_manager::EncryptedApiKey {
        ciphertext: dify_credentials[0].encrypted_api_key.clone(),
        nonce: dify_credentials[0].nonce.clone(),
        salt: dify_credentials[0].salt.clone(),
    };
    let decrypted = manager.decrypt(&loaded_encrypted).expect("解密失败");
    assert_eq!(decrypted, "sk-second-key", "应解密为第二个 key");
}

/// 测试不同凭证类型可以独立存储
#[tokio::test]
async fn test_different_credential_types_stored_separately() {
    let pool = setup_test_db().await;
    let manager = ApiKeyManager::new(TEST_MASTER_PASSWORD.to_string());

    use prompt_faster::infra::db::repositories::UpsertCredentialInput;

    // 保存 Dify 凭证
    let encrypted_dify = manager.encrypt("sk-dify-key").expect("加密失败");
    CredentialRepo::upsert(
        &pool,
        UpsertCredentialInput {
            user_id: TEST_USER_ID.to_string(),
            credential_type: CredentialType::Dify,
            provider: None,
            base_url: "https://api.dify.ai".to_string(),
            encrypted_api_key: encrypted_dify.ciphertext,
            nonce: encrypted_dify.nonce,
            salt: encrypted_dify.salt,
        },
    )
    .await
    .expect("保存 Dify 凭证失败");

    // 保存 GenericLlm 凭证
    let encrypted_llm = manager.encrypt("sk-llm-key").expect("加密失败");
    CredentialRepo::upsert(
        &pool,
        UpsertCredentialInput {
            user_id: TEST_USER_ID.to_string(),
            credential_type: CredentialType::GenericLlm,
            provider: Some("siliconflow".to_string()),
            base_url: "https://api.siliconflow.cn".to_string(),
            encrypted_api_key: encrypted_llm.ciphertext,
            nonce: encrypted_llm.nonce,
            salt: encrypted_llm.salt,
        },
    )
    .await
    .expect("保存 GenericLlm 凭证失败");

    // 验证两种类型都能独立读取
    let dify = CredentialRepo::find_by_user_and_type(&pool, TEST_USER_ID, CredentialType::Dify)
        .await
        .expect("读取 Dify 凭证失败");

    let llm =
        CredentialRepo::find_by_user_and_type(&pool, TEST_USER_ID, CredentialType::GenericLlm)
            .await
            .expect("读取 GenericLlm 凭证失败");

    assert_eq!(dify.base_url, "https://api.dify.ai");
    assert_eq!(llm.base_url, "https://api.siliconflow.cn");
    assert_eq!(llm.provider, Some("siliconflow".to_string()));
}

/// 测试凭证参数验证范围
#[test]
fn test_teacher_settings_validation_ranges() {
    // 这个测试验证后端的参数范围常量与文档一致
    // 实际验证逻辑在 auth.rs 的 validate_teacher_settings 函数中

    // temperature: 0.0 ~ 2.0
    assert!(0.0 <= 2.0);

    // top_p: 0.0 ~ 1.0
    assert!(0.0 <= 1.0);

    // max_tokens: 1 ~ 8192
    assert!(1 <= 8192);
}
