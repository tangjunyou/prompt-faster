use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::Duration;

use axum::{Router, extract::State, routing::get, response::IntoResponse, Json};
use serde_json::json;
use tokio::net::TcpListener;

use prompt_faster::domain::models::ConnectivityStatus;
use prompt_faster::infra::external::connectivity::check_connectivity_status;
use prompt_faster::infra::external::llm_client;

async fn models_handler(State(counter): State<Arc<AtomicUsize>>) -> impl IntoResponse {
    let attempt = counter.fetch_add(1, Ordering::SeqCst);
    if attempt < 2 {
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    Json(json!({"data": [{"id": "model-a"}]}))
}

#[tokio::test]
async fn llm_list_models_retries_after_timeout() {
    let counter = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/v1/models", get(models_handler))
        .with_state(counter.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("绑定端口失败");
    let addr = listener.local_addr().expect("读取地址失败");
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("启动服务失败");
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(200))
        .connect_timeout(Duration::from_millis(200))
        .build()
        .expect("创建测试客户端失败");

    let base_url = format!("http://{}", addr);
    let result = llm_client::list_models(&client, &base_url, "test-key", "siliconflow", "cid-1")
        .await
        .expect("重试后应成功");

    assert_eq!(result, vec!["model-a".to_string()]);
    assert!(counter.load(Ordering::SeqCst) >= 3);
    assert!(matches!(
        check_connectivity_status().await,
        ConnectivityStatus::Online
    ));
}
