fn main() {
    println!("cargo:rerun-if-changed=resources");
    tauri_build::build()
}
