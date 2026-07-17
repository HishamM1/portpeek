import assert from "node:assert/strict";
import test from "node:test";
import { brandSlug, groupPorts, isDatabase } from "./ports.js";

const port = (id, pid, number, processName = "node.exe") => ({
  id,
  pid,
  port: number,
  processName,
  displayName: null,
  command: null,
  executablePath: null,
  framework: null,
});

const portWithFramework = (id, pid, number, processName, frameworkName) => ({
  id,
  pid,
  port: number,
  processName,
  displayName: null,
  command: null,
  executablePath: null,
  framework: { name: frameworkName, confidence: "high", source: "process_name" },
});

test("groups listeners by PID and recognizes desktop apps", () => {
  const groups = groupPorts([port("a", 42, 3000), port("b", 42, 3001), port("c", 77, 4000)]);
  assert.deepEqual(groups.map((group) => group.map((item) => item.port)), [[3000, 3001], [4000]]);
  assert.equal(brandSlug(port("cursor", 9, 3000, "Cursor.exe")), "cursor");
  assert.equal(brandSlug(port("tailscale", 10, 41112, "tailscaled.exe")), "tailscale");
  assert.equal(brandSlug(port("postgres", 11, 5432, "postgres.exe")), "postgresql");
  assert.equal(isDatabase(port("sqlserver", 12, 1433, "sqlservr.exe")), true);
});

test("framework brand takes priority over app brand", () => {
  assert.equal(brandSlug(portWithFramework("a", 1, 3000, "java.exe", "Spring Boot")), "spring");
  assert.equal(brandSlug(portWithFramework("b", 2, 3001, "nginx.exe", "Nginx")), "nginx");
  assert.equal(brandSlug(portWithFramework("c", 3, 3002, "python.exe", "FastAPI")), "fastapi");
});

test("maps all Phase 1 framework brands correctly", () => {
  const cases = [
    ["Next.js", "nextdotjs"],
    ["Nuxt", "nuxt"],
    ["SvelteKit", "svelte"],
    ["Vite", "vite"],
    ["React", "react"],
    ["Vue", "vuedotjs"],
    ["Node.js", "nodedotjs"],
    ["Django", "django"],
    ["FastAPI", "fastapi"],
    ["Rails", "rubyonrails"],
    ["Laravel", "laravel"],
    ["Express", "express"],
    ["NestJS", "nestjs"],
    ["Flask", "flask"],
    ["Spring Boot", "spring"],
    [".NET", "dotnet"],
    ["ASP.NET", "dotnet"],
    ["Java", "openjdk"],
    ["Nginx", "nginx"],
    ["OpenSSH", "openssh"],
    ["Caddy", "caddy"],
    ["Apache HTTP Server", "apache"],
    ["IIS Express", "iis"],
    ["Uvicorn", "uvicorn"],
    ["Gunicorn", "gunicorn"],
    ["Streamlit", "streamlit"],
    ["Jupyter", "jupyter"],
    ["Astro", "astro"],
    ["Deno", "deno"],
    ["Bun", "bun"],
    ["Phoenix", "phoenixframework"],
    ["Elixir", "elixir"],
    ["SQL Server", "microsoftsqlserver"],
    ["RabbitMQ", "rabbitmq"],
    ["Apache Kafka", "apachekafka"],
    ["MinIO", "minio"],
    ["Prometheus", "prometheus"],
    ["Grafana", "grafana"],
    ["HashiCorp Vault", "vault"],
    ["Traefik", "traefik"],
    ["PostgreSQL", "postgresql"],
    ["Redis", "redis"],
    ["MySQL", "mysql"],
  ];
  for (const [name, slug] of cases) {
    assert.equal(brandSlug(portWithFramework("x", 0, 8080, "node.exe", name)), slug, `mismatch for ${name}`);
  }
});

test("maps all Phase 2 framework brands correctly", () => {
  const cases = [
    ["Jetty", "jetty"],
    ["WildFly", "wildfly"],
    ["Ktor", "ktor"],
    ["Play Framework", "playframework"],
    ["Grails", "grails"],
    ["Fastify", "fastify"],
    ["Hapi", "hapi"],
    ["Koa", "koajs"],
    ["AdonisJS", "adonisjs"],
    ["Remix", "remix"],
    ["Symfony", "symfony"],
    ["WordPress", "wordpress"],
    ["Sinatra", "sinatra"],
    ["Axum", "axum"],
    ["Actix Web", "actix"],
    ["Rocket", "rocket"],
    ["Warp", "warp"],
    ["Gin", "gin"],
    ["Fiber", "gofiber"],
    ["Echo", "echo"],
    ["Dart", "dart"],
    ["Flutter", "flutter"],
    ["gRPC", "grpc"],
    ["Jaeger", "jaeger"],
    ["OpenTelemetry Collector", "opentelemetry"],
    ["Envoy", "envoy"],
  ];
  for (const [name, slug] of cases) {
    assert.equal(brandSlug(portWithFramework("x", 0, 8080, "node.exe", name)), slug, `mismatch for ${name}`);
  }
});

test("returns null for unknown framework", () => {
  assert.equal(brandSlug(portWithFramework("x", 0, 8080, "node.exe", "UnknownFrameworkXYZ")), null);
});
