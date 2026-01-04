//! API Key 加密管理
//! 使用 AES-GCM + Argon2 派生密钥 (NFR9)
//!
//! # 安全说明
//! - 使用 Argon2id 从用户登录密码（会话内存副本）派生 256 位密钥
//! - 使用 AES-256-GCM 加密 API Key
//! - 每次加密生成随机 12 字节 nonce
//! - 每条记录使用独立的 salt
//!
//! # 向后兼容
//! - 如设置 `MASTER_PASSWORD`，可用于解密旧数据（旧版本使用全局 master_password 派生）
//! - 新写入数据始终使用用户登录密码派生（UnlockContext）

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::{Argon2, Params, Version};
use rand::RngCore;
use thiserror::Error;
use zeroize::Zeroizing;

use crate::shared::log_sanitizer::sanitize_api_key;

/// AES-GCM nonce 长度（12 字节 = 96 bits，AES-GCM 标准）
pub const NONCE_LENGTH: usize = 12;

/// Argon2 salt 长度（16 字节）
pub const SALT_LENGTH: usize = 16;

/// 派生密钥长度（32 字节 = 256 bits，用于 AES-256）
const KEY_LENGTH: usize = 32;

/// API Key 加解密错误
#[derive(Error, Debug)]
pub enum ApiKeyError {
    #[error("密钥派生失败: {0}")]
    KeyDerivationFailed(String),

    #[error("加密失败: {0}")]
    EncryptionFailed(String),

    #[error("解密失败: {0}")]
    DecryptionFailed(String),

    #[error("无效的 nonce 长度: 期望 {expected}，实际 {actual}")]
    InvalidNonceLength { expected: usize, actual: usize },

    #[error("无效的 salt 长度: 期望 {expected}，实际 {actual}")]
    InvalidSaltLength { expected: usize, actual: usize },
}

/// 加密后的 API Key 数据
#[derive(Debug, Clone)]
pub struct EncryptedApiKey {
    /// 加密后的密文
    pub ciphertext: Vec<u8>,
    /// 12 字节随机 nonce
    pub nonce: Vec<u8>,
    /// 16 字节随机 salt（用于 Argon2 派生密钥）
    pub salt: Vec<u8>,
}

/// API Key 管理器
///
/// 负责 API Key 的加密和解密操作。
/// 使用 Argon2id 从用户登录密码派生密钥，AES-256-GCM 加密数据。
pub struct ApiKeyManager {
    /// legacy 主密码（仅用于向后兼容解密旧数据）
    legacy_master_password: Option<Zeroizing<Vec<u8>>>,
}

impl ApiKeyManager {
    /// 创建新的 API Key 管理器
    ///
    /// # Arguments
    /// * `legacy_master_password` - legacy 主密码（可选，仅用于向后兼容解密旧数据）
    pub fn new(legacy_master_password: Option<String>) -> Self {
        Self {
            legacy_master_password: legacy_master_password
                .filter(|s| !s.is_empty())
                .map(|s| Zeroizing::new(s.into_bytes())),
        }
    }

    /// 加密 API Key
    ///
    /// # Arguments
    /// * `user_password` - 用户登录密码（来自 UnlockContext，仅存在内存）
    /// * `api_key` - 明文 API Key
    ///
    /// # Returns
    /// 加密后的数据（密文 + nonce + salt）
    pub fn encrypt(
        &self,
        user_password: &[u8],
        api_key: &str,
    ) -> Result<EncryptedApiKey, ApiKeyError> {
        tracing::debug!(
            api_key = %sanitize_api_key(api_key),
            "加密 API Key"
        );

        // 生成随机 salt
        let salt = generate_random_bytes(SALT_LENGTH);

        // 生成随机 nonce
        let nonce_bytes = generate_random_bytes(NONCE_LENGTH);

        // 派生密钥
        let key = derive_key(user_password, &salt)?;

        // 创建 AES-GCM 密码器
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key[..]));
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密
        let ciphertext = cipher
            .encrypt(nonce, api_key.as_bytes())
            .map_err(|e| ApiKeyError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedApiKey {
            ciphertext,
            nonce: nonce_bytes,
            salt,
        })
    }

    /// 解密 API Key
    ///
    /// # Arguments
    /// * `user_password` - 用户登录密码（来自 UnlockContext，仅存在内存）
    /// * `encrypted` - 加密后的数据
    ///
    /// # Returns
    /// 解密后的明文 API Key
    pub fn decrypt(
        &self,
        user_password: &[u8],
        encrypted: &EncryptedApiKey,
    ) -> Result<String, ApiKeyError> {
        let plaintext = self.decrypt_bytes(user_password, encrypted)?;
        let api_key = String::from_utf8(plaintext.to_vec())
            .map_err(|e| ApiKeyError::DecryptionFailed(e.to_string()))?;

        tracing::debug!(
            api_key = %sanitize_api_key(&api_key),
            "解密 API Key 成功"
        );

        Ok(api_key)
    }

    /// 解密 API Key（字节形式，使用 zeroize 控制明文生命周期）
    pub fn decrypt_bytes(
        &self,
        user_password: &[u8],
        encrypted: &EncryptedApiKey,
    ) -> Result<Zeroizing<Vec<u8>>, ApiKeyError> {
        validate_encrypted(encrypted)?;

        // 主路径：用户登录密码派生
        if let Ok(key) = derive_key(user_password, &encrypted.salt) {
            if let Ok(plaintext) = decrypt_with_key(&key[..], encrypted) {
                return Ok(Zeroizing::new(plaintext));
            }
        }

        // 向后兼容：legacy MASTER_PASSWORD
        if let Some(legacy_pwd) = &self.legacy_master_password {
            let key = derive_key(legacy_pwd.as_slice(), &encrypted.salt)?;
            let plaintext = decrypt_with_key(&key[..], encrypted)
                .map_err(|e| ApiKeyError::DecryptionFailed(e.to_string()))?;
            return Ok(Zeroizing::new(plaintext));
        }

        Err(ApiKeyError::DecryptionFailed(
            "使用用户密码与 legacy 主密码均解密失败".to_string(),
        ))
    }
}

fn validate_encrypted(encrypted: &EncryptedApiKey) -> Result<(), ApiKeyError> {
    if encrypted.nonce.len() != NONCE_LENGTH {
        return Err(ApiKeyError::InvalidNonceLength {
            expected: NONCE_LENGTH,
            actual: encrypted.nonce.len(),
        });
    }

    if encrypted.salt.len() != SALT_LENGTH {
        return Err(ApiKeyError::InvalidSaltLength {
            expected: SALT_LENGTH,
            actual: encrypted.salt.len(),
        });
    }

    Ok(())
}

/// 使用 Argon2id 从指定密码派生密钥（Drop 时自动清零）
fn derive_key(password: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LENGTH]>, ApiKeyError> {
    let params = Params::new(65536, 3, 4, Some(KEY_LENGTH))
        .map_err(|e| ApiKeyError::KeyDerivationFailed(e.to_string()))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; KEY_LENGTH]);
    argon2
        .hash_password_into(password, salt, &mut key[..])
        .map_err(|e| ApiKeyError::KeyDerivationFailed(e.to_string()))?;

    Ok(key)
}

fn decrypt_with_key(key: &[u8], encrypted: &EncryptedApiKey) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&encrypted.nonce);
    cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
}

/// 生成指定长度的随机字节
fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let manager = ApiKeyManager::new(None);
        let user_password = b"test_password";
        let api_key = "sk-1234567890abcdef";

        let encrypted = manager.encrypt(user_password, api_key).unwrap();
        let decrypted = manager.decrypt(user_password, &encrypted).unwrap();

        assert_eq!(api_key, decrypted);
    }

    #[test]
    fn test_different_salt_produces_different_ciphertext() {
        let manager = ApiKeyManager::new(None);
        let user_password = b"test_password";
        let api_key = "sk-1234567890abcdef";

        let encrypted1 = manager.encrypt(user_password, api_key).unwrap();
        let encrypted2 = manager.encrypt(user_password, api_key).unwrap();

        // 不同的 salt 应该产生不同的密文
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        assert_ne!(encrypted1.salt, encrypted2.salt);
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
    }

    #[test]
    fn test_wrong_password_fails_decryption() {
        let manager1 = ApiKeyManager::new(None);
        let manager2 = ApiKeyManager::new(None);
        let password1 = b"password1";
        let password2 = b"password2";
        let api_key = "sk-1234567890abcdef";

        let encrypted = manager1.encrypt(password1, api_key).unwrap();
        let result = manager2.decrypt(password2, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_length() {
        let manager = ApiKeyManager::new(None);
        let encrypted = manager.encrypt(b"test_password", "test_key").unwrap();

        assert_eq!(encrypted.nonce.len(), NONCE_LENGTH);
    }

    #[test]
    fn test_salt_length() {
        let manager = ApiKeyManager::new(None);
        let encrypted = manager.encrypt(b"test_password", "test_key").unwrap();

        assert_eq!(encrypted.salt.len(), SALT_LENGTH);
    }

    #[test]
    fn test_invalid_nonce_length() {
        let manager = ApiKeyManager::new(None);
        let encrypted = EncryptedApiKey {
            ciphertext: vec![1, 2, 3],
            nonce: vec![1, 2, 3], // 错误的长度
            salt: vec![0u8; SALT_LENGTH],
        };

        let result = manager.decrypt(b"test_password", &encrypted);
        assert!(matches!(
            result,
            Err(ApiKeyError::InvalidNonceLength { .. })
        ));
    }

    #[test]
    fn test_invalid_salt_length() {
        let manager = ApiKeyManager::new(None);
        let encrypted = EncryptedApiKey {
            ciphertext: vec![1, 2, 3],
            nonce: vec![0u8; NONCE_LENGTH],
            salt: vec![1, 2, 3], // 错误的长度
        };

        let result = manager.decrypt(b"test_password", &encrypted);
        assert!(matches!(result, Err(ApiKeyError::InvalidSaltLength { .. })));
    }

    #[test]
    fn test_encrypted_data_is_not_plaintext() {
        let manager = ApiKeyManager::new(None);
        let api_key = "sk-1234567890abcdef";

        let encrypted = manager.encrypt(b"test_password", api_key).unwrap();

        // 密文不应该包含明文
        let ciphertext_str = String::from_utf8_lossy(&encrypted.ciphertext);
        assert!(!ciphertext_str.contains(api_key));
    }
}
