//! 密码哈希与校验服务
//! 使用 Argon2 算法进行密码哈希（AC #1, #6）
//!
//! # 安全约束
//! - 密码仅以 Argon2 PHC 字符串格式存储
//! - 日志不得包含明文密码（AC #6）
//! - 错误信息只输出通用描述（AC #3）

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use thiserror::Error;

/// 密码服务错误
#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("密码哈希失败")]
    HashError,

    #[error("密码哈希格式无效")]
    InvalidHashFormat,

    #[error("用户名或密码错误")]
    VerificationFailed,
}

/// 密码服务
/// 提供 Argon2 密码哈希与校验功能
pub struct PasswordService;

impl PasswordService {
    /// 对密码进行 Argon2 哈希
    ///
    /// # 参数
    /// - `password`: 明文密码
    ///
    /// # 返回
    /// - 成功时返回 PHC 格式的哈希字符串（以 `$argon2` 开头）
    /// - 失败时返回 `PasswordError::HashError`
    ///
    /// # 示例
    /// ```ignore
    /// let hash = PasswordService::hash_password("my_password")?;
    /// // hash 格式: $argon2id$v=19$m=65536,t=3,p=1$...
    /// ```
    pub fn hash_password(password: &str) -> Result<String, PasswordError> {
        let password_bytes = password.as_bytes();
        let salt = SaltString::generate(&mut OsRng);

        // 使用 Argon2id v19 默认参数
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password_bytes, &salt)
            .map_err(|_| PasswordError::HashError)?
            .to_string();

        Ok(password_hash)
    }

    /// 校验密码是否匹配
    ///
    /// # 参数
    /// - `password`: 明文密码
    /// - `stored_hash`: 存储的 PHC 格式哈希字符串
    ///
    /// # 返回
    /// - 成功时返回 `Ok(())`
    /// - 密码不匹配或哈希格式无效时返回 `PasswordError::VerificationFailed`
    ///
    /// # 注意
    /// 返回统一的错误信息，不区分"哈希格式无效"与"密码不匹配"（AC #3）
    pub fn verify_password(password: &str, stored_hash: &str) -> Result<(), PasswordError> {
        let parsed_hash =
            PasswordHash::new(stored_hash).map_err(|_| PasswordError::VerificationFailed)?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| PasswordError::VerificationFailed)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_produces_phc_string() {
        let hash = PasswordService::hash_password("test_password").unwrap();
        // PHC 字符串应该以 $argon2 开头
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_correct_password() {
        let password = "correct_password";
        let hash = PasswordService::hash_password(password).unwrap();

        assert!(PasswordService::verify_password(password, &hash).is_ok());
    }

    #[test]
    fn test_verify_wrong_password() {
        let hash = PasswordService::hash_password("correct_password").unwrap();

        let result = PasswordService::verify_password("wrong_password", &hash);
        assert!(result.is_err());
        assert!(matches!(result, Err(PasswordError::VerificationFailed)));
    }

    #[test]
    fn test_verify_invalid_hash_format() {
        let result = PasswordService::verify_password("any_password", "invalid_hash");
        assert!(result.is_err());
        assert!(matches!(result, Err(PasswordError::VerificationFailed)));
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let hash1 = PasswordService::hash_password("password1").unwrap();
        let hash2 = PasswordService::hash_password("password2").unwrap();
        // 不同密码应该产生不同的哈希
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_produces_different_hashes() {
        // 由于使用随机 salt，相同密码每次哈希结果应该不同
        let hash1 = PasswordService::hash_password("same_password").unwrap();
        let hash2 = PasswordService::hash_password("same_password").unwrap();
        assert_ne!(hash1, hash2);

        // 但两个哈希都应该能验证通过
        assert!(PasswordService::verify_password("same_password", &hash1).is_ok());
        assert!(PasswordService::verify_password("same_password", &hash2).is_ok());
    }
}
