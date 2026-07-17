use std::{fs, path::{Path, PathBuf}};

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
    ProcessName,
    Unknown,
}

pub fn detect(
    process_name: Option<&str>,
    command: Option<&str>,
    root: Option<&Path>,
) -> Option<FrameworkDetection> {
    let pname = process_name.unwrap_or_default();
    let cmd = command.unwrap_or_default();

    detect_by_unique_process_name(pname)
        .or_else(|| {
            root.and_then(detect_package)
                .filter(|d| runtime_matches(pname, cmd, &d.name))
        })
        .or_else(|| {
            root.and_then(|r| detect_config(r))
                .filter(|d| runtime_matches(pname, cmd, &d.name))
        })
        .or_else(|| {
            command
                .and_then(detect_command)
                .filter(|d| runtime_matches(pname, cmd, &d.name))
        })
        .or_else(|| detect_generic_runtime(pname, cmd, root))
}

fn runtime_matches(process_name: &str, command: &str, framework: &str) -> bool {
    let pn = process_name.to_ascii_lowercase();
    let cmd = command.to_ascii_lowercase();

    if pn.contains(&framework.to_ascii_lowercase())
        || cmd.contains(&framework.to_ascii_lowercase())
    {
        return true;
    }

    match framework {
        "Next.js" | "Nuxt" | "SvelteKit" | "Vite" | "React" | "Vue" | "Svelte"
        | "Node.js" | "Express" | "NestJS" | "Astro" | "Fastify" | "Hapi" | "Koa"
        | "AdonisJS" | "Remix" => {
            ["node", "bun", "deno", "npm", "pnpm", "yarn"]
                .iter()
                .any(|r| pn.contains(r) || cmd.contains(r))
        }
        "Django" | "FastAPI" | "Flask" | "Uvicorn" | "Gunicorn" | "Streamlit"
        | "Jupyter" | "Hypercorn" | "Gradio" | "Python" => {
            pn.contains("python") || cmd.contains("python")
        }
        "Rails" | "Sinatra" | "Puma" | "Ruby" => {
            pn.contains("ruby") || cmd.contains("ruby") || cmd.contains("rails")
        }
        "Laravel" | "PHP" | "Symfony" | "CodeIgniter" | "WordPress" => {
            pn.contains("php") || cmd.contains("php")
        }
        "Rust" | "Axum" | "Actix Web" | "Rocket" | "Warp" => {
            pn.contains("cargo") || cmd.contains("cargo") || cmd.contains("rustc")
        }
        "Go" | "Gin" | "Fiber" | "Echo" => {
            pn.contains("go.exe") || cmd.contains("go run")
        }
        "Java" | "Spring Boot" | "Apache Tomcat" | "Quarkus" | "Micronaut"
        | "Jetty" | "WildFly" | "Ktor" | "Play Framework" | "Grails" => {
            pn.contains("java") || cmd.contains("java") || cmd.contains("javaw")
        }
        ".NET" | "ASP.NET" => {
            pn.contains("dotnet") || cmd.contains("dotnet") || pn.contains("iisexpress")
        }
        "Phoenix" | "Elixir" => {
            cmd.contains("elixir") || cmd.contains("iex") || cmd.contains("mix")
                || pn.contains("erl.exe") || pn.contains("beam")
        }
        "Dart" | "Flutter" => {
            pn.contains("dart") || cmd.contains("dart") || pn.contains("flutter")
                || cmd.contains("flutter")
        }
        "gRPC" => cmd.contains("grpc") || pn.contains("grpc"),
        _ => false,
    }
}

fn detect_by_unique_process_name(process_name: &str) -> Option<FrameworkDetection> {
    let name = process_name.to_ascii_lowercase();
    let name = name.strip_suffix(".exe").unwrap_or(&name);

    let result = match name {
        "nginx" => Some(("Nginx", FrameworkConfidence::High)),
        "httpd" | "apache" => Some(("Apache HTTP Server", FrameworkConfidence::High)),
        "sshd" => Some(("OpenSSH", FrameworkConfidence::High)),
        "caddy" => Some(("Caddy", FrameworkConfidence::High)),
        "traefik" => Some(("Traefik", FrameworkConfidence::High)),
        "envoy" => Some(("Envoy", FrameworkConfidence::High)),
        "iisexpress" | "iisexpresstray" => {
            Some(("IIS Express", FrameworkConfidence::High))
        }
        "w3wp" => Some(("ASP.NET", FrameworkConfidence::High)),
        "sqlservr" => Some(("SQL Server", FrameworkConfidence::High)),
        "rabbitmq-server" | "rabbitmq" => {
            Some(("RabbitMQ", FrameworkConfidence::High))
        }
        "kafka-server-start" => Some(("Apache Kafka", FrameworkConfidence::High)),
        "minio" => Some(("MinIO", FrameworkConfidence::High)),
        "prometheus" => Some(("Prometheus", FrameworkConfidence::High)),
        "grafana" => Some(("Grafana", FrameworkConfidence::High)),
        "vault" => Some(("HashiCorp Vault", FrameworkConfidence::High)),
        "consul" => Some(("HashiCorp Consul", FrameworkConfidence::High)),
        "memcached" => Some(("Memcached", FrameworkConfidence::High)),
        "jaeger" => Some(("Jaeger", FrameworkConfidence::High)),
        "otelcol" => Some(("OpenTelemetry Collector", FrameworkConfidence::High)),
        "tableplus" => Some(("TablePlus", FrameworkConfidence::High)),
        "antigravity" | "agy" => Some(("Antigravity", FrameworkConfidence::High)),
        "mailpit" => Some(("Mailpit", FrameworkConfidence::High)),
        "mailhog" => Some(("MailHog", FrameworkConfidence::High)),
        "localstack" => Some(("LocalStack", FrameworkConfidence::High)),
        "mysqld" => Some(("MySQL", FrameworkConfidence::High)),
        "mariadbd" => Some(("MariaDB", FrameworkConfidence::High)),
        "mongod" | "mongos" => Some(("MongoDB", FrameworkConfidence::High)),
        "redis-server" => Some(("Redis", FrameworkConfidence::High)),
        "postgres" => Some(("PostgreSQL", FrameworkConfidence::High)),
        "cockroach" => Some(("CockroachDB", FrameworkConfidence::High)),
        "influxd" => Some(("InfluxDB", FrameworkConfidence::High)),
        "elasticsearch" => Some(("Elasticsearch", FrameworkConfidence::High)),
        "neo4j" => Some(("Neo4j", FrameworkConfidence::High)),
        "cassandra" => Some(("Cassandra", FrameworkConfidence::High)),
        "clickhouse" => Some(("ClickHouse", FrameworkConfidence::High)),
        _ => None,
    };

    result.map(|(framework, confidence)| {
        found(framework, confidence, FrameworkDetectionSource::ProcessName)
    })
}

fn detect_package(root: &Path) -> Option<FrameworkDetection> {
    let bytes = read_small_file(&root.join("package.json"))?;
    let package: Value = serde_json::from_slice(&bytes).ok()?;
    detect_specific_package_json(&package).or_else(|| {
        Some(found(
            "Node.js",
            FrameworkConfidence::Medium,
            FrameworkDetectionSource::PackageJson,
        ))
    })
}

fn detect_specific_package_json(package: &Value) -> Option<FrameworkDetection> {
    let deps_keys = ["dependencies", "devDependencies", "peerDependencies"]
        .into_iter()
        .filter_map(|key| package.get(key)?.as_object())
        .flat_map(|deps| deps.keys());
    let has = |target: &str| deps_keys.clone().any(|k| k == target);

    let name = if has("next") {
        "Next.js"
    } else if has("nuxt") {
        "Nuxt"
    } else if has("astro") {
        "Astro"
    } else if has("@sveltejs/kit") {
        "SvelteKit"
    } else if has("vite") {
        "Vite"
    } else if has("react") {
        "React"
    } else if has("vue") {
        "Vue"
    } else if has("svelte") {
        "Svelte"
    } else if has("express") {
        "Express"
    } else if has("@nestjs/core") || has("@nestjs/common") {
        "NestJS"
    } else if has("fastify") {
        "Fastify"
    } else if has("@hapi/hapi") {
        "Hapi"
    } else if has("koa") {
        "Koa"
    } else if has("@adonisjs/core") {
        "AdonisJS"
    } else if has("@remix-run/react") || has("@remix-run/node") {
        "Remix"
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
    let flat_candidates: &[(&[&str], &str)] = &[
        (&["next.config.js", "next.config.mjs", "next.config.ts"][..], "Next.js"),
        (&["nuxt.config.js", "nuxt.config.ts"][..], "Nuxt"),
        (&["svelte.config.js", "svelte.config.ts"][..], "SvelteKit"),
        (&["vite.config.js", "vite.config.ts"][..], "Vite"),
        (&["astro.config.mjs", "astro.config.js", "astro.config.ts"][..], "Astro"),
        (&["manage.py"][..], "Django"),
        (&["artisan"][..], "Laravel"),
    ];

    for (files, name) in flat_candidates {
        if files.iter().any(|file| root.join(file).exists()) {
            return Some(found(
                name,
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(pyproject) = read_small_file(&root.join("pyproject.toml")) {
        let text = String::from_utf8_lossy(&pyproject).to_ascii_lowercase();
        if text.contains("fastapi") || text.contains("uvicorn") {
            let name = if text.contains("fastapi") {
                "FastAPI"
            } else {
                "Uvicorn"
            };
            return Some(found(
                name,
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("flask") {
            return Some(found(
                "Flask",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("streamlit") {
            return Some(found(
                "Streamlit",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("gradio") {
            return Some(found(
                "Gradio",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("gunicorn") {
            return Some(found(
                "Gunicorn",
                FrameworkConfidence::Medium,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(gemfile) = root
        .join("Gemfile")
        .exists()
        .then(|| read_small_file(&root.join("Gemfile")))
        .flatten()
    {
        let text = String::from_utf8_lossy(&gemfile).to_ascii_lowercase();
        if text.contains("sinatra") {
            return Some(found(
                "Sinatra",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("puma") {
            return Some(found(
                "Puma",
                FrameworkConfidence::Medium,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("rails") {
            return Some(found(
                "Rails",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(composer) = root
        .join("composer.json")
        .exists()
        .then(|| read_small_file(&root.join("composer.json")))
        .flatten()
    {
        if let Ok(decoded) = serde_json::from_slice::<Value>(&composer) {
            let deps = decoded
                .get("require")
                .and_then(|v| v.as_object())
                .into_iter()
                .flat_map(|m| m.keys())
                .chain(
                    decoded
                        .get("require-dev")
                        .and_then(|v| v.as_object())
                        .into_iter()
                        .flat_map(|m| m.keys()),
                );
            let has = |target: &str| deps.clone().any(|k| k == target);

            if has("laravel/framework") || has("laravel") {
                return Some(found(
                    "Laravel",
                    FrameworkConfidence::High,
                    FrameworkDetectionSource::ConfigFile,
                ));
            }
            if has("symfony/framework-bundle") || has("symfony/symfony") {
                return Some(found(
                    "Symfony",
                    FrameworkConfidence::High,
                    FrameworkDetectionSource::ConfigFile,
                ));
            }
            if has("codeigniter/framework") {
                return Some(found(
                    "CodeIgniter",
                    FrameworkConfidence::High,
                    FrameworkDetectionSource::ConfigFile,
                ));
            }
            if has("wp-cli/wp-cli") {
                return Some(found(
                    "WordPress",
                    FrameworkConfidence::Medium,
                    FrameworkDetectionSource::ConfigFile,
                ));
            }
        }
    }

    if let Some(requirements) = root
        .join("requirements.txt")
        .exists()
        .then(|| read_small_file(&root.join("requirements.txt")))
        .flatten()
    {
        let text = String::from_utf8_lossy(&requirements).to_ascii_lowercase();
        if text.contains("flask") {
            return Some(found(
                "Flask",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("fastapi") {
            return Some(found(
                "FastAPI",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("django") {
            return Some(found(
                "Django",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(cargo_str) = root
        .join("Cargo.toml")
        .exists()
        .then(|| read_small_file(&root.join("Cargo.toml")))
        .flatten()
    {
        let text = String::from_utf8_lossy(&cargo_str).to_ascii_lowercase();
        if text.contains("axum") {
            return Some(found(
                "Axum",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("actix-web") || text.contains("actix_web") {
            return Some(found(
                "Actix Web",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("rocket") {
            return Some(found(
                "Rocket",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("warp") {
            return Some(found(
                "Warp",
                FrameworkConfidence::Medium,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(modfile) = root
        .join("go.mod")
        .exists()
        .then(|| read_small_file(&root.join("go.mod")))
        .flatten()
    {
        let text = String::from_utf8_lossy(&modfile).to_ascii_lowercase();
        if text.contains("gin-gonic/gin") || text.contains("gin ") {
            return Some(found(
                "Gin",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("gofiber/fiber") || text.contains("fiber ") {
            return Some(found(
                "Fiber",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("labstack/echo") || text.contains("echo ") {
            return Some(found(
                "Echo",
                FrameworkConfidence::Medium,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(mix) = root
        .join("mix.exs")
        .exists()
        .then(|| read_small_file(&root.join("mix.exs")))
        .flatten()
    {
        let text = String::from_utf8_lossy(&mix).to_ascii_lowercase();
        if text.contains("phoenix") {
            return Some(found(
                "Phoenix",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(pubspec) = root
        .join("pubspec.yaml")
        .exists()
        .then(|| read_small_file(&root.join("pubspec.yaml")))
        .flatten()
        .or_else(|| {
            root.join("pubspec.yml")
                .exists()
                .then(|| read_small_file(&root.join("pubspec.yml")))
                .flatten()
        })
    {
        let text = String::from_utf8_lossy(&pubspec).to_ascii_lowercase();
        if text.contains("flutter") {
            return Some(found(
                "Flutter",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("shelf") {
            return Some(found(
                "Dart",
                FrameworkConfidence::Medium,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(pom) = root
        .join("pom.xml")
        .exists()
        .then(|| read_small_file(&root.join("pom.xml")))
        .flatten()
    {
        let text = String::from_utf8_lossy(&pom).to_ascii_lowercase();
        if text.contains("spring-boot") {
            return Some(found(
                "Spring Boot",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("quarkus") {
            return Some(found(
                "Quarkus",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("micronaut") {
            return Some(found(
                "Micronaut",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("tomcat") {
            return Some(found(
                "Apache Tomcat",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("jetty") {
            return Some(found(
                "Jetty",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("wildfly") || text.contains("jboss") {
            return Some(found(
                "WildFly",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("play") {
            return Some(found(
                "Play Framework",
                FrameworkConfidence::Medium,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("grails") {
            return Some(found(
                "Grails",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(gradle) = root
        .join("build.gradle")
        .exists()
        .then(|| read_small_file(&root.join("build.gradle")))
        .flatten()
        .or_else(|| {
            root.join("build.gradle.kts")
                .exists()
                .then(|| read_small_file(&root.join("build.gradle.kts")))
                .flatten()
        })
    {
        let text = String::from_utf8_lossy(&gradle).to_ascii_lowercase();
        if text.contains("spring-boot") || text.contains("spring boot") {
            return Some(found(
                "Spring Boot",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("quarkus") {
            return Some(found(
                "Quarkus",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("micronaut") {
            return Some(found(
                "Micronaut",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("grails") {
            return Some(found(
                "Grails",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
        if text.contains("ktor") {
            return Some(found(
                "Ktor",
                FrameworkConfidence::High,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if let Some(app_props) = root
        .join("application.properties")
        .exists()
        .then(|| read_small_file(&root.join("application.properties")))
        .flatten()
    {
        let text = String::from_utf8_lossy(&app_props).to_ascii_lowercase();
        if text.contains("spring") {
            return Some(found(
                "Spring Boot",
                FrameworkConfidence::Medium,
                FrameworkDetectionSource::ConfigFile,
            ));
        }
    }

    if root.join("application.yml").exists() || root.join("application.yaml").exists() {
        return Some(found(
            "Spring Boot",
            FrameworkConfidence::Medium,
            FrameworkDetectionSource::ConfigFile,
        ));
    }

    if let Some(csproj) = glob_first(root, "*.csproj") {
        if let Some(bytes) = read_small_file(&csproj) {
            let text = String::from_utf8_lossy(&bytes).to_ascii_lowercase();
            if text.contains("microsoft.aspnetcore") || text.contains("kestrel") {
                return Some(found(
                    "ASP.NET",
                    FrameworkConfidence::High,
                    FrameworkDetectionSource::ConfigFile,
                ));
            }
        }
        return Some(found(
            ".NET",
            FrameworkConfidence::Medium,
            FrameworkDetectionSource::ConfigFile,
        ));
    }

    let generic = if root.join("Cargo.toml").exists() {
        Some("Rust")
    } else if root.join("go.mod").exists() {
        Some("Go")
    } else if root.join("composer.json").exists() {
        Some("PHP")
    } else if root.join("mix.exs").exists() {
        Some("Elixir")
    } else if root.join("pubspec.yaml").exists() || root.join("pubspec.yml").exists() {
        Some("Dart")
    } else if root.join("Program.cs").exists() {
        Some(".NET")
    } else if root.join("pom.xml").exists()
        || root.join("build.gradle").exists()
        || root.join("build.gradle.kts").exists()
    {
        Some("Java")
    } else if root.join("Gemfile").exists() {
        Some("Ruby")
    } else if root.join("requirements.txt").exists() {
        Some("Python")
    } else {
        None
    };

    generic.map(|name| found(name, FrameworkConfidence::Medium, FrameworkDetectionSource::ConfigFile))
}

fn detect_command(command: &str) -> Option<FrameworkDetection> {
    let cmd = command.to_ascii_lowercase();

    if cmd.contains("spring-boot") || cmd.contains("spring boot") || cmd.contains("springboot") {
        return Some(found("Spring Boot", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("mix phx") || cmd.contains("mix phoenix") {
        return Some(found("Phoenix", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("actix-web") || cmd.contains("actix_web") {
        return Some(found("Actix Web", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("opentelemetry") || cmd.contains("otel-collector") {
        return Some(found("OpenTelemetry Collector", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("kafka-server-start") || cmd.contains("kafka-server-start.bat") {
        return Some(found("Apache Kafka", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("spring ") || cmd.contains("spring.") {
        return Some(found("Spring Boot", FrameworkConfidence::Low, FrameworkDetectionSource::Command));
    }

    if cmd.contains("uvicorn") {
        return Some(found("Uvicorn", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("gunicorn") {
        return Some(found("Gunicorn", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("flask") {
        return Some(found("Flask", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("fastapi") {
        return Some(found("FastAPI", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("streamlit") {
        return Some(found("Streamlit", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("jupyter") {
        return Some(found("Jupyter", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("hypercorn") {
        return Some(found("Hypercorn", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("gradio") {
        return Some(found("Gradio", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("manage.py") || cmd.contains("django-admin") {
        return Some(found("Django", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("django") {
        return Some(found("Django", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }

    if cmd.contains("express") {
        return Some(found("Express", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("nest ") || cmd.contains("nestjs") {
        return Some(found("NestJS", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("astro") {
        return Some(found("Astro", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("fastify") {
        return Some(found("Fastify", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("@hapi/hapi") || cmd.contains(" hapi ") {
        return Some(found("Hapi", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains(" koa ") {
        return Some(found("Koa", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("adonis") {
        return Some(found("AdonisJS", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("@remix-run") || cmd.contains("remix ") {
        return Some(found("Remix", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }

    if cmd.contains("quarkus") {
        return Some(found("Quarkus", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("micronaut") {
        return Some(found("Micronaut", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("tomcat") {
        return Some(found("Apache Tomcat", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("jetty") {
        return Some(found("Jetty", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("wildfly") || cmd.contains("jboss") {
        return Some(found("WildFly", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains(" ktor ") {
        return Some(found("Ktor", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("play framework") || cmd.contains("playframework") {
        return Some(found("Play Framework", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("grails") {
        return Some(found("Grails", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }

    if cmd.contains("puma") {
        return Some(found("Puma", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("sinatra") {
        return Some(found("Sinatra", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }

    if cmd.contains("symfony") {
        return Some(found("Symfony", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("codeigniter") {
        return Some(found("CodeIgniter", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("artisan") || cmd.contains("laravel") {
        return Some(found("Laravel", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("wp ") {
        return Some(found("WordPress", FrameworkConfidence::Low, FrameworkDetectionSource::Command));
    }

    if cmd.contains("axum") {
        return Some(found("Axum", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("rocket") {
        return Some(found("Rocket", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains(" warp ") {
        return Some(found("Warp", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }

    if cmd.contains("gin ") {
        return Some(found("Gin", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("fiber ") {
        return Some(found("Fiber", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains(" echo ") {
        return Some(found("Echo", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }

    if cmd.contains("dart run") || cmd.contains("dart ") {
        return Some(found("Dart", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("flutter") {
        return Some(found("Flutter", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }

    if cmd.contains("grpc") {
        return Some(found("gRPC", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("jaeger") {
        return Some(found("Jaeger", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("envoy") {
        return Some(found("Envoy", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("traefik") {
        return Some(found("Traefik", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }

    if cmd.contains("rails") {
        return Some(found("Rails", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("next") {
        return Some(found("Next.js", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("nuxt") {
        return Some(found("Nuxt", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("vite") {
        return Some(found("Vite", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("svelte-kit") || cmd.contains("sveltekit") {
        return Some(found("SvelteKit", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("cargo run") {
        return Some(found("Rust", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("cargo") {
        return Some(found("Rust", FrameworkConfidence::Low, FrameworkDetectionSource::Command));
    }
    if cmd.contains("go run") {
        return Some(found("Go", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("dotnet run") || cmd.contains("dotnet ") {
        return Some(found(".NET", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("java -jar") || cmd.contains("java ") {
        return Some(found("Java", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("php -s") || cmd.contains("php ") {
        return Some(found("PHP", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("python -m") || cmd.contains("python3 -m") || cmd.contains("python ") {
        return Some(found("Python", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("bun") {
        return Some(found("Bun", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("deno") {
        return Some(found("Deno", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.starts_with("node ") || cmd.contains(" node ") {
        return Some(found("Node.js", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("ruby ") {
        return Some(found("Ruby", FrameworkConfidence::Medium, FrameworkDetectionSource::Command));
    }
    if cmd.contains("mix ") {
        return Some(found("Elixir", FrameworkConfidence::Low, FrameworkDetectionSource::Command));
    }
    if cmd.contains("iex ") {
        return Some(found("Elixir", FrameworkConfidence::Low, FrameworkDetectionSource::Command));
    }

    None
}

fn detect_generic_runtime(
    process_name: &str,
    command: &str,
    root: Option<&Path>,
) -> Option<FrameworkDetection> {
    let pn = process_name.to_ascii_lowercase();
    let cmd = command.to_ascii_lowercase();

    if pn.contains("java") || cmd.contains("java") {
        return Some(found("Java", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("dotnet") || cmd.contains("dotnet") {
        return Some(found(".NET", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("python") || cmd.contains("python") {
        return Some(found("Python", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("node") || cmd.contains("node ") || cmd.contains(" node") {
        return Some(found("Node.js", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("ruby") || cmd.contains("ruby") {
        return Some(found("Ruby", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("php") || cmd.contains("php") {
        return Some(found("PHP", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("go.exe") || cmd.contains("go run") {
        return Some(found("Go", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("cargo") || cmd.contains("cargo") || cmd.contains("rustc") {
        return Some(found("Rust", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("dart") || cmd.contains("dart") {
        return Some(found("Dart", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if pn.contains("erl.exe") || cmd.contains("elixir") || cmd.contains("iex") || cmd.contains("mix ") {
        return Some(found("Elixir", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }

    let root = root?;

    if root.join("Cargo.toml").exists() {
        return Some(found("Rust", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if root.join("go.mod").exists() {
        return Some(found("Go", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if root.join("package.json").exists() {
        return Some(found("Node.js", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if root.join("Gemfile").exists() {
        return Some(found("Ruby", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if root.join("composer.json").exists() {
        return Some(found("PHP", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        return Some(found("Python", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if root.join("pom.xml").exists() || root.join("build.gradle").exists() {
        return Some(found("Java", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if glob_first(root, "*.csproj").is_some() || root.join("Program.cs").exists() {
        return Some(found(".NET", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if root.join("mix.exs").exists() {
        return Some(found("Elixir", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }
    if root.join("pubspec.yaml").exists() || root.join("pubspec.yml").exists() {
        return Some(found("Dart", FrameworkConfidence::Low, FrameworkDetectionSource::Unknown));
    }

    None
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

fn glob_first(root: &Path, pattern: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(root).ok()?;
    for entry in entries {
        let path = entry.ok()?.path();
        if path.is_file() {
            if let Some(name) = path.file_name()?.to_str() {
                if pattern_match(name, pattern) {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn pattern_match(name: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        name.starts_with(prefix)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        name.ends_with(suffix)
    } else {
        name == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_frameworks_from_process_name() {
        assert_eq!(
            detect_by_unique_process_name("nginx.exe").unwrap().name,
            "Nginx"
        );
        assert_eq!(
            detect_by_unique_process_name("sshd.exe").unwrap().name,
            "OpenSSH"
        );
        assert_eq!(
            detect_by_unique_process_name("caddy").unwrap().name,
            "Caddy"
        );
        assert_eq!(
            detect_by_unique_process_name("dotnet.exe"),
            None
        );
        assert_eq!(
            detect_by_unique_process_name("node.exe"),
            None
        );
    }

    #[test]
    fn detects_existing_from_package_json() {
        let package = serde_json::json!({ "dependencies": { "next": "15" } });
        assert_eq!(
            detect_specific_package_json(&package).unwrap().name,
            "Next.js"
        );

        let pkg = serde_json::json!({ "dependencies": { "@nestjs/core": "10" } });
        assert_eq!(detect_specific_package_json(&pkg).unwrap().name, "NestJS");

        let pkg = serde_json::json!({ "dependencies": { "express": "4" } });
        assert_eq!(detect_specific_package_json(&pkg).unwrap().name, "Express");

        let pkg = serde_json::json!({ "dependencies": { "astro": "4" } });
        assert_eq!(detect_specific_package_json(&pkg).unwrap().name, "Astro");

        let pkg = serde_json::json!({ "devDependencies": { "fastify": "4" } });
        assert_eq!(detect_specific_package_json(&pkg).unwrap().name, "Fastify");
    }

    #[test]
    fn detects_express_via_command() {
        let result = detect_command("node node_modules/.bin/express-generator");
        assert_eq!(result.unwrap().name, "Express");
    }

    #[test]
    fn detects_uvicorn_via_command() {
        let result = detect_command("python -m uvicorn app:api --reload");
        assert_eq!(result.unwrap().name, "Uvicorn");
    }

    #[test]
    fn detects_flask_via_command() {
        let result = detect_command("python -m flask run");
        assert_eq!(result.unwrap().name, "Flask");
    }

    #[test]
    fn runtime_matches_node_frameworks() {
        assert!(runtime_matches("node.exe", "node app.js", "Next.js"));
        assert!(runtime_matches("bun.exe", "bun run", "Express"));
        assert!(runtime_matches("deno.exe", "deno task", "Astro"));
        assert!(!runtime_matches("java.exe", "java -jar app.jar", "Next.js"));
    }

    #[test]
    fn runtime_matches_python_frameworks() {
        assert!(runtime_matches("python.exe", "python -m flask run", "Flask"));
        assert!(runtime_matches("python3.exe", "python3 -m uvicorn app:api", "Uvicorn"));
        assert!(runtime_matches("python.exe", "python manage.py runserver", "Django"));
        assert!(!runtime_matches("node.exe", "node app.js", "Flask"));
    }

    #[test]
    fn runtime_matches_jvm_frameworks() {
        assert!(runtime_matches("java.exe", "java -jar app.jar", "Spring Boot"));
        assert!(runtime_matches("javaw.exe", "javaw -jar app.jar", "Java"));
        assert!(!runtime_matches("node.exe", "node app.js", "Spring Boot"));
    }

    #[test]
    fn runtime_matches_dotnet() {
        assert!(runtime_matches("dotnet.exe", "dotnet run", ".NET"));
        assert!(runtime_matches("iisexpress.exe", "", "ASP.NET"));
        assert!(!runtime_matches("python.exe", "", ".NET"));
    }

    #[test]
    fn runtime_matches_go_frameworks() {
        assert!(runtime_matches("go.exe", "go run main.go", "Go"));
        assert!(runtime_matches("go.exe", "go run main.go", "Gin"));
        assert!(!runtime_matches("node.exe", "node app.js", "Gin"));
    }

    #[test]
    fn generic_runtime_fallback_java() {
        let detection = detect_generic_runtime("java.exe", "", None).unwrap();
        assert_eq!(detection.name, "Java");
        assert_eq!(detection.confidence, FrameworkConfidence::Low);
    }

    #[test]
    fn generic_runtime_fallback_node() {
        let detection = detect_generic_runtime("node.exe", "", None).unwrap();
        assert_eq!(detection.name, "Node.js");
    }

    #[test]
    fn full_detection_pipeline_java_process() {
        let detection = detect(Some("java.exe"), Some("java -jar app.jar --spring-boot"), None);
        assert_eq!(detection.unwrap().name, "Spring Boot");
    }

    #[test]
    fn full_detection_pipeline_node_process() {
        let detection = detect(Some("node.exe"), Some("node node_modules/.bin/next dev"), None);
        assert_eq!(detection.unwrap().name, "Next.js");
    }

    #[test]
    fn full_detection_pipeline_nginx() {
        let detection = detect(Some("nginx.exe"), None, None).unwrap();
        assert_eq!(detection.name, "Nginx");
    }

    #[test]
    fn no_false_positive_for_unrelated_process() {
        let result = detect(Some("explorer.exe"), None, None);
        assert!(result.is_none());
    }

    #[test]
    fn command_detects_astro() {
        let result = detect_command("npx astro dev");
        assert_eq!(result.unwrap().name, "Astro");
    }

    #[test]
    fn command_detects_storybook() {
        let result = detect_command("storybook dev -p 6006");
        assert!(result.is_none());
    }

    #[test]
    fn command_detects_spring_boot() {
        let result = detect_command("java -jar app.jar --spring-boot");
        assert_eq!(result.unwrap().name, "Spring Boot");
    }

    #[test]
    fn command_detects_vite() {
        let result = detect_command("npx vite --port 3000");
        assert_eq!(result.unwrap().name, "Vite");
    }

    #[test]
    fn command_detects_gunicorn() {
        let result = detect_command("gunicorn app:wsgi -b 0.0.0.0:8000");
        assert_eq!(result.unwrap().name, "Gunicorn");
    }

    #[test]
    fn command_detects_streamlit() {
        let result = detect_command("streamlit run app.py");
        assert_eq!(result.unwrap().name, "Streamlit");
    }

    #[test]
    fn command_detects_jupyter() {
        let result = detect_command("jupyter notebook");
        assert_eq!(result.unwrap().name, "Jupyter");
    }

    #[test]
    fn command_detects_phoenix() {
        let result = detect_command("mix phx.server");
        assert_eq!(result.unwrap().name, "Phoenix");
    }

    #[test]
    fn command_detects_rails() {
        let result = detect_command("rails server -p 3000");
        assert_eq!(result.unwrap().name, "Rails");
    }

    #[test]
    fn command_precedence_specific_over_generic() {
        let result = detect_command("java -jar app.jar --spring-boot");
        assert_eq!(result.unwrap().name, "Spring Boot");
    }

    #[test]
    fn process_name_matches_infrastructure_tools() {
        assert_eq!(detect_by_unique_process_name("redis-server.exe").unwrap().name, "Redis");
        assert_eq!(detect_by_unique_process_name("postgres.exe").unwrap().name, "PostgreSQL");
        assert_eq!(detect_by_unique_process_name("mysqld.exe").unwrap().name, "MySQL");
        assert_eq!(detect_by_unique_process_name("prometheus.exe").unwrap().name, "Prometheus");
        assert_eq!(detect_by_unique_process_name("grafana.exe").unwrap().name, "Grafana");
        assert_eq!(detect_by_unique_process_name("vault.exe").unwrap().name, "HashiCorp Vault");
        assert_eq!(detect_by_unique_process_name("consul.exe").unwrap().name, "HashiCorp Consul");
        assert_eq!(detect_by_unique_process_name("traefik.exe").unwrap().name, "Traefik");
        assert_eq!(detect_by_unique_process_name("envoy.exe").unwrap().name, "Envoy");
    }

    #[test]
    fn process_name_returns_none_for_generic_runtimes() {
        assert!(detect_by_unique_process_name("node.exe").is_none());
        assert!(detect_by_unique_process_name("python.exe").is_none());
        assert!(detect_by_unique_process_name("java.exe").is_none());
        assert!(detect_by_unique_process_name("dotnet.exe").is_none());
        assert!(detect_by_unique_process_name("ruby.exe").is_none());
        assert!(detect_by_unique_process_name("php.exe").is_none());
    }

    #[test]
    fn runtime_matches_without_exe() {
        assert!(runtime_matches("python", "python app.py", "Flask"));
        assert!(runtime_matches("node", "node app.js", "Express"));
    }

    #[test]
    fn detects_postgres_by_process_name() {
        let detection = detect(Some("postgres.exe"), None, None).unwrap();
        assert_eq!(detection.name, "PostgreSQL");
    }

    #[test]
    fn detect_returns_none_for_empty_input() {
        assert!(detect(None, None, None).is_none());
    }

    #[test]
    fn config_detection_prefers_specific_frameworks_over_generic_markers() {
        let root = std::env::temp_dir().join(format!("portpeek-framework-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        fs::write(root.join("pom.xml"), "spring-boot-starter-web").unwrap();
        assert_eq!(detect_config(&root).unwrap().name, "Spring Boot");

        fs::remove_file(root.join("pom.xml")).unwrap();
        fs::write(root.join("composer.json"), r#"{"require":{"symfony/framework-bundle":"*"}}"#).unwrap();
        assert_eq!(detect_config(&root).unwrap().name, "Symfony");

        fs::remove_dir_all(root).unwrap();
    }
}
