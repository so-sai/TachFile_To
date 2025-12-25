use crate::ipc::protocol::{ErrorPayload, IpcMessage, MessageType, SuccessPayload};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use uuid::Uuid;

/// Kiểu dữ liệu trả về cho caller (Result hoặc Error)
#[derive(Debug)]
pub enum RouterResponse {
    Success(Value),
    Error(ErrorPayload),
}

/// Bộ định tuyến tin nhắn IPC
#[derive(Clone)]
pub struct MessageRouter {
    // Map: Request ID -> Channel gửi kết quả lại cho caller
    // Dùng oneshot vì mỗi request chỉ có 1 response
    pending_requests: Arc<Mutex<HashMap<Uuid, oneshot::Sender<RouterResponse>>>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Đăng ký một request mới cần chờ phản hồi
    pub fn register_request(&self, req_id: Uuid) -> oneshot::Receiver<RouterResponse> {
        let (tx, rx) = oneshot::channel();
        let mut map = self.pending_requests.lock().unwrap();
        map.insert(req_id, tx);
        rx
    }

    /// Xử lý tin nhắn đến (Inbound Message từ Python)
    pub async fn dispatch(&self, json_str: &str) {
        // 1. Parse Envelope (giữ payload dưới dạng Value để linh hoạt)
        let msg_result: Result<IpcMessage<Value>, _> = serde_json::from_str(json_str);

        match msg_result {
            Ok(msg) => {
                self.handle_message(msg).await;
            }
            Err(e) => {
                eprintln!("❌ ROUTER: Failed to parse incoming JSON: {}", e);
                eprintln!("   Raw: {}", json_str);
            }
        }
    }

    /// Logic xử lý từng loại tin nhắn
    async fn handle_message(&self, msg: IpcMessage<Value>) {
        match msg.msg_type {
            // Trường hợp 1: Python trả về Success
            MessageType::ResSuccess => {
                // Parse payload thành SuccessPayload để lấy req_id
                if let Ok(payload) = serde_json::from_value::<SuccessPayload<Value>>(msg.payload) {
                    self.resolve_request(payload.req_id, RouterResponse::Success(payload.data));
                }
            }

            // Trường hợp 2: Python trả về Error
            MessageType::ResError => {
                if let Ok(payload) = serde_json::from_value::<ErrorPayload>(msg.payload) {
                    self.resolve_request(payload.req_id, RouterResponse::Error(payload));
                }
            }

            // Trường hợp 3: Handshake (Python báo sẵn sàng)
            MessageType::ResHandshake => {
                println!(
                    "✅ ROUTER: Received Handshake from Worker: {:?}",
                    msg.payload
                );
                // TODO: Trigger event "System Ready"
            }

            // Trường hợp 4: Progress (Xử lý file lớn)
            MessageType::ResProgress => {
                // TODO: Forward to Frontend via Tauri Event
                println!("⏳ ROUTER: Progress update: {:?}", msg.payload);
            }

            _ => {
                eprintln!("⚠️ ROUTER: Unhandled message type: {:?}", msg.msg_type);
            }
        }
    }

    /// Tìm và trả kết quả cho người chờ
    fn resolve_request(&self, req_id: Uuid, response: RouterResponse) {
        let mut map = self.pending_requests.lock().unwrap();
        if let Some(tx) = map.remove(&req_id) {
            if let Err(_) = tx.send(response) {
                eprintln!("⚠️ ROUTER: Caller dropped receiver for request {}", req_id);
            }
        } else {
            // Có thể là timeout đã xảy ra trước đó
            eprintln!(
                "⚠️ ROUTER: Received response for unknown/expired Request ID: {}",
                req_id
            );
        }
    }
}

// --- UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_dispatch_success_flow() {
        let router = MessageRouter::new();
        let req_id = Uuid::new_v4();

        // 1. Giả lập việc gửi request và chờ đợi
        let rx = router.register_request(req_id);

        // 2. Giả lập Python trả về JSON Success
        let response_json = json!({
            "protocol_v": "1.0.0",
            "msg_id": Uuid::new_v4(),
            "timestamp": 123456789,
            "type": "RES_SUCCESS",
            "payload": {
                "req_id": req_id,
                "data": { "foo": "bar" },
                "metadata": {}
            }
        })
        .to_string();

        // 3. Dispatch
        router.dispatch(&response_json).await;

        // 4. Verify kết quả nhận được
        match rx.await {
            Ok(RouterResponse::Success(val)) => {
                assert_eq!(val["foo"], "bar");
            }
            _ => panic!("Expected Success response"),
        }
    }
}
