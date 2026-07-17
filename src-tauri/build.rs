fn main() {
    // Create dummy CLI binary target to break circular dependency if not present
    let target_dir = std::path::Path::new("target/release");
    if !target_dir.exists() {
        let _ = std::fs::create_dir_all(target_dir);
    }
    let dummy_path = target_dir.join("portpeek-cli.exe");
    if !dummy_path.exists() {
        let _ = std::fs::write(&dummy_path, "");
    }

    tauri_build::build()
}
