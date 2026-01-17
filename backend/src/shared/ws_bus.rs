//! WebSocket 事件总线（内存广播）

use std::sync::{Arc, OnceLock};

use tokio::sync::broadcast;

/// 广播通道容量（避免慢消费者阻塞）
const WS_BUS_CAPACITY: usize = 256;

#[derive(Clone)]
pub struct WsEventBus {
    sender: broadcast::Sender<String>,
}

impl WsEventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(WS_BUS_CAPACITY);
        Self { sender }
    }

    /// 订阅事件
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    /// 发布事件（序列化后的 JSON 文本）
    pub fn publish(&self, message: String) {
        let _ = self.sender.send(message);
    }
}

static WS_BUS: OnceLock<Arc<WsEventBus>> = OnceLock::new();

/// 获取全局 WS 事件总线
pub fn global_ws_bus() -> Arc<WsEventBus> {
    WS_BUS
        .get_or_init(|| Arc::new(WsEventBus::new()))
        .clone()
}
