use std::path::{Path, PathBuf};

const PROJECT_MARKERS: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "manage.py",
    "Gemfile",
    "composer.json",
    "artisan",
];

pub fn find_root(working_directory: Option<&str>) -> Option<PathBuf> {
    let mut directory = PathBuf::from(working_directory?);

    for _ in 0..=6 {
        if is_project_root(&directory) {
            return Some(directory);
        }
        if !directory.pop() {
            break;
        }
    }

    None
}

fn is_project_root(directory: &Path) -> bool {
    PROJECT_MARKERS
        .iter()
        .any(|marker| directory.join(marker).exists())
}
