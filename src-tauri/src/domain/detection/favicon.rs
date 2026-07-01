use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager};

const MAX_FAVICON_BYTES: u64 = 2 * 1024 * 1024;
const CANDIDATES: &[&str] = &[
    "public/favicon.ico",
    "public/favicon.png",
    "public/favicon.svg",
    "static/favicon.ico",
    "static/favicon.png",
    "static/favicon.svg",
    "src/app/favicon.ico",
    "src/app/favicon.png",
    "favicon.ico",
    "favicon.png",
    "favicon.svg",
];

pub fn cache_project_favicon(app: &AppHandle, root: &Path) -> Option<String> {
    let source = CANDIDATES
        .iter()
        .map(|candidate| root.join(candidate))
        .find(|candidate| valid_favicon(candidate))?;
    let cache_dir = app.path().app_cache_dir().ok()?.join("favicons");
    fs::create_dir_all(&cache_dir).ok()?;
    let target = cached_path(&cache_dir, &source);

    if should_copy(&source, &target) {
        fs::copy(source, &target).ok()?;
    }

    Some(target.to_string_lossy().into_owned())
}

fn should_copy(source: &Path, target: &Path) -> bool {
    let Ok(source) = fs::metadata(source) else {
        return false;
    };
    let Ok(target) = fs::metadata(target) else {
        return true;
    };
    source.len() != target.len() || source.modified().ok() > target.modified().ok()
}

fn valid_favicon(path: &Path) -> bool {
    path.is_file()
        && fs::metadata(path)
            .map(|metadata| metadata.len() <= MAX_FAVICON_BYTES)
            .unwrap_or(false)
}

fn cached_path(cache_dir: &Path, source: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("ico");
    cache_dir.join(format!("{:x}.{extension}", hasher.finish()))
}
