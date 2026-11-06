fn main() {
    // Rebuild the native resources when the application's icons change.
    println!("cargo:rerun-if-changed=icons");
    tauri_build::build()
}
