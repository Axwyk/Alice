use crate::core::demo_chat::{ChatError, ChatRequest, ChatResponse, reply};
use std::{thread, time::Duration};

#[tauri::command]
pub async fn send_demo_message(request: ChatRequest) -> Result<ChatResponse, ChatError> {
    let request_id = request.request_id.clone();
    // Demo latency makes the real IPC operation's pending state observable.
    // It is confined to this demonstration command, not the domain service.
    match tauri::async_runtime::spawn_blocking(move || {
        thread::sleep(Duration::from_millis(1200));
        reply(request)
    })
    .await
    {
        Ok(result) => result,
        Err(_) => Err(ChatError::internal(request_id)),
    }
}
