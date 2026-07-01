use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkDetection {
    pub name: String,
    pub confidence: FrameworkConfidence,
    pub source: FrameworkDetectionSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FrameworkConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkDetectionSource {
    Command,
    PackageJson,
    ConfigFile,
    HttpProbe,
    Unknown,
}

pub fn detect(
    process_name: Option<&str>,
    command: Option<&str>,
    root: Option<&Path>,
) -> Option<FrameworkDetection> {
    root.and_then(detect_package)
        .filter(|detection| runtime_matches(process_name, command, &detection.name))
        .or_else(|| {
            root.and_then(detect_config)
                .filter(|detection| runtime_matches(process_name, command, &detection.name))
        })
        .or_else(|| command.and_then(detect_command))
}

fn runtime_matches(process_name: Option<&str>, command: Option<&str>, framework: &str) -> bool {
    let process = process_name.unwrap_or_default().to_ascii_lowercase();
    let command = command.unwrap_or_default().to_ascii_lowercase();
    match framework {
        "Next.js" | "Nuxt" | "SvelteKit" | "Vite" | "React" | "Vue" | "Svelte" | "Node" => {
            ["node", "bun", "deno", "npm", "pnpm", "yarn"]
                .iter()
                .any(|runtime| process.contains(runtime) || command.contains(runtime))
        }
        "Django" | "FastAPI" => process.contains("python") || command.contains("python"),
        "Rails" => process.contains("ruby") || command.contains("rails"),
        "Laravel" | "PHP" => process.contains("php") || command.contains("php"),
        "Rust" => process.contains("cargo") || command.contains("\\target\\"),
        "Go" => process == "go.exe" || command.contains("go run"),
        _ => false,
    }
}

fn detect_package(root: &Path) -> Option<FrameworkDetection> {
    let bytes = read_small_file(&root.join("package.json"))?;
    let package: Value = serde_json::from_slice(&bytes).ok()?;
    detect_package_json(&package).or_else(|| {
        Some(found(
            "Node",
            FrameworkConfidence::Medium,
            FrameworkDetectionSource::PackageJson,
        ))
    })
}

fn detect_package_json(package: &Value) -> Option<FrameworkDetection> {
    let dependencies = ["dependencies", "devDependencies"]
        .into_iter()
        .filter_map(|key| package.get(key)?.as_object())
        .flat_map(|dependencies| dependencies.keys());
    let names: Vec<&str> = dependencies.map(String::as_str).collect();

    let name = if names.contains(&"next") {
        "Next.js"
    } else if names.contains(&"nuxt") {
        "Nuxt"
    } else if names.contains(&"@sveltejs/kit") {
        "SvelteKit"
    } else if names.contains(&"vite") {
        "Vite"
    } else if names.contains(&"react") {
        "React"
    } else if names.contains(&"vue") {
        "Vue"
    } else if names.contains(&"svelte") {
        "Svelte"
    } else {
        return None;
    };

    Some(found(
        name,
        FrameworkConfidence::High,
        FrameworkDetectionSource::PackageJson,
    ))
}

fn detect_config(root: &Path) -> Option<FrameworkDetection> {
    let candidates = [
        (
            &["next.config.js", "next.config.mjs", "next.config.ts"][..],
            "Next.js",
        ),
        (&["nuxt.config.js", "nuxt.config.ts"][..], "Nuxt"),
        (&["svelte.config.js", "svelte.config.ts"][..], "SvelteKit"),
        (&["vite.config.js", "vite.config.ts"][..], "Vite"),
        (&["manage.py"][..], "Django"),
        (&["artisan"][..], "Laravel"),
        (&["Cargo.toml"][..], "Rust"),
        (&["go.mod"][..], "Go"),
        (&["composer.json"][..], "PHP"),
    ];

    for (files, name) in candidates {
        if files.iter().any(|file| root.join(file).exists()) {
            return Some(found(
                name,
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(pyproject) = read_small_file(&root.join("pyproject.toml")) {
        if String::from_utf8_lossy(&pyproject)
            .to_ascii_lowercase()
            .contains("fastapi")
        {
            return Some(found(
                "FastAPI",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if root.join("Gemfile").exists() {
        let gemfile = read_small_file(&root.join("Gemfile"))?;
        if String::from_utf8_lossy(&gemfile)
            .to_ascii_lowercase()
            .contains("rails")
        {
            return Some(found(
                "Rails",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }
    None
}

fn detect_command(command: &str) -> Option<FrameworkDetection> {
    let command = command.to_ascii_lowercase();
    let name = if command.contains("next") {
        "Next.js"
    } else if command.contains("nuxt") {
        "Nuxt"
    } else if command.contains("vite") {
        "Vite"
    } else if command.contains("svelte-kit") || command.contains("sveltekit") {
        "SvelteKit"
    } else if command.contains("uvicorn") || command.contains("fastapi") {
        "FastAPI"
    } else if command.contains("manage.py") || command.contains("django") {
        "Django"
    } else if command.contains("rails") {
        "Rails"
    } else if command.contains("artisan") || command.contains("laravel") {
        "Laravel"
    } else if command.contains("cargo run") {
        "Rust"
    } else if command.contains("go run") {
        "Go"
    } else if command.contains("php -s") {
        "PHP"
    } else if command.starts_with("node ") || command.contains(" node ") {
        "Node"
    } else {
        return None;
    };

    Some(found(
        name,
        FrameworkConfidence::Medium,
        FrameworkDetectionSource::Command,
    ))
}

fn found(
    name: &str,
    confidence: FrameworkConfidence,
    source: FrameworkDetectionSource,
) -> FrameworkDetection {
    FrameworkDetection {
        name: name.into(),
        confidence,
        source,
    }
}

fn read_small_file(path: &Path) -> Option<Vec<u8>> {
    let metadata = fs::metadata(path).ok()?;
    (metadata.len() <= 2 * 1024 * 1024)
        .then(|| fs::read(path).ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_framework_from_package_and_command() {
        let package = serde_json::json!({ "dependencies": { "next": "15" } });
        assert_eq!(detect_package_json(&package).unwrap().name, "Next.js");
        assert_eq!(
            detect_command("python -m uvicorn app:api").unwrap().name,
            "FastAPI"
        );
        assert!(!runtime_matches(Some("java.exe"), None, "SvelteKit"));
        assert!(runtime_matches(Some("node.exe"), None, "SvelteKit"));
    }
}
