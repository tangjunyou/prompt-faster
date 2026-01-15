use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::core::traits::TeacherModel;

/// 示例 TeacherModel：确定性、不出网，适合作为扩展模板与单测注入。
#[derive(Debug, Clone)]
pub struct ExampleTeacherModel {
    response: Arc<str>,
    delay: Option<Duration>,
}

impl ExampleTeacherModel {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: Arc::from(response.into()),
            delay: None,
        }
    }

    pub fn new_default() -> Self {
        // 默认返回“通过”结果（JSON），不回显 prompt 原文。
        Self::new("{\"passed\":true,\"score\":1,\"confidence\":1}")
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

#[async_trait]
impl TeacherModel for ExampleTeacherModel {
    async fn generate(&self, _prompt: &str) -> anyhow::Result<String> {
        if let Some(d) = self.delay {
            sleep(d).await;
        }
        Ok(self.response.to_string())
    }

    async fn generate_stream(&self, _prompt: &str) -> anyhow::Result<mpsc::Receiver<String>> {
        let (tx, rx) = mpsc::channel(8);
        let response = self.response.clone();
        let delay = self.delay;
        tokio::spawn(async move {
            if let Some(d) = delay {
                sleep(d).await;
            }
            let _ = tx.send(response.to_string()).await;
        });
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn example_teacher_model_generate_is_deterministic() {
        let tm = ExampleTeacherModel::new("{\"passed\":true,\"score\":1}").with_delay(Duration::from_millis(1));
        let out = tm.generate("PROMPT_SHOULD_NOT_LEAK").await.unwrap();
        assert_eq!(out, "{\"passed\":true,\"score\":1}");
        assert!(!out.contains("PROMPT_SHOULD_NOT_LEAK"));
    }
}

