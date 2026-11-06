mod commands;
mod core;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::send_demo_message])
        .run(tauri::generate_context!())
        .expect("Failed to start the Alice desktop application");
}
