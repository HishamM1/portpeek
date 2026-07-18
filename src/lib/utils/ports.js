/** @typedef {import("$lib/types/port").PortItem} PortItem */

/** @type {[string[], string][]} */
const appBrands = [
  [["cursor.exe", "cursor editor"], "cursor"],
  [["warp.exe", "warp terminal"], "warp"],
  [["tailscale", "tailscaled", "tailscale-ipn"], "tailscale"],
  [["code.exe", "visual studio code"], "https://code.visualstudio.com/favicon.ico"],
  [["devenv.exe", "visual studio"], "https://visualstudio.microsoft.com/wp-content/uploads/2021/10/Product-Icon.svg"],
  [["idea64.exe", "intellij"], "intellijidea"],
  [["webstorm64.exe", "webstorm"], "webstorm"],
  [["pycharm64.exe", "pycharm"], "pycharm"],
  [["phpstorm64.exe", "phpstorm"], "phpstorm"],
  [["rider64.exe", "jetbrains rider"], "rider"],
  [["clion64.exe", "clion"], "clion"],
  [["goland64.exe", "goland"], "goland"],
  [["studio64.exe", "android studio"], "androidstudio"],
  [["sublime_text.exe", "sublime text"], "sublimetext"],
  [["docker desktop", "docker.exe", "dockerd.exe"], "docker"],
  [["postgres.exe", "postgresql", "pg_ctl.exe", "pgadmin4.exe"], "postgresql"],
  [["mariadbd.exe", "mariadb"], "mariadb"],
  [["mysqld.exe", "mysql server"], "mysql"],
  [["mongod.exe", "mongos.exe", "mongodb"], "mongodb"],
  [["redis-server.exe", "redis server"], "redis"],
  [["sqlite3.exe"], "sqlite"],
  [["cockroach.exe", "cockroachdb"], "cockroachlabs"],
  [["influxd.exe", "influxdb"], "influxdb"],
  [["elasticsearch"], "elasticsearch"],
  [["neo4j"], "neo4j"],
  [["cassandra"], "apachecassandra"],
  [["clickhouse"], "clickhouse"],
  [["dbeaver.exe"], "dbeaver"],
  [["datagrip64.exe", "datagrip"], "datagrip"],
  [["bun.exe"], "bun"],
  [["deno.exe"], "deno"],
  [["dotnet.exe"], "dotnet"],
  [["java.exe", "javaw.exe"], "openjdk"],
  [["nginx.exe"], "nginx"],
];

const frameworkBrands = new Map([
  ["next.js", "nextdotjs"],
  ["nuxt", "nuxt"],
  ["sveltekit", "svelte"],
  ["svelte", "svelte"],
  ["vite", "vite"],
  ["react", "react"],
  ["vue", "vuedotjs"],
  ["node.js", "nodedotjs"],
  ["node", "nodedotjs"],
  ["django", "django"],
  ["fastapi", "fastapi"],
  ["rails", "rubyonrails"],
  ["laravel", "laravel"],
  ["php", "php"],
  ["go", "go"],
  ["rust", "rust"],
  // Phase 1 — Runtimes & frameworks
  [".net", "dotnet"],
  ["asp.net", "dotnet"],
  ["java", "openjdk"],
  ["spring boot", "spring"],
  ["apache tomcat", "apachetomcat"],
  ["quarkus", "quarkus"],
  ["micronaut", "micronaut"],
  ["flask", "flask"],
  ["express", "express"],
  ["nestjs", "nestjs"],
  ["iis express", "iis"],
  ["openssh", "openssh"],
  ["nginx", "nginx"],
  ["caddy", "caddy"],
  ["apache http server", "apache"],
  ["uvicorn", "uvicorn"],
  ["gunicorn", "gunicorn"],
  ["streamlit", "streamlit"],
  ["jupyter", "jupyter"],
  ["astro", "astro"],
  ["deno", "deno"],
  ["bun", "bun"],
  ["puma", "puma"],
  ["phoenix", "phoenixframework"],
  ["elixir", "elixir"],
  ["antigravity", "antigravity"],
  ["tableplus", "tableplus"],
  // Phase 1 — Infrastructure
  ["sql server", "microsoftsqlserver"],
  ["rabbitmq", "rabbitmq"],
  ["apache kafka", "apachekafka"],
  ["minio", "minio"],
  ["localstack", "localstack"],
  ["mailpit", "mailpit"],
  ["mailhog", "mailhog"],
  ["memcached", "memcached"],
  ["hashicorp vault", "vault"],
  ["hashicorp consul", "consul"],
  ["prometheus", "prometheus"],
  ["grafana", "grafana"],
  ["traefik", "traefik"],
  // Phase 1 — Databases
  ["mysql", "mysql"],
  ["mariadb", "mariadb"],
  ["mongodb", "mongodb"],
  ["postgresql", "postgresql"],
  ["redis", "redis"],
  ["cockroachdb", "cockroachlabs"],
  ["influxdb", "influxdb"],
  ["elasticsearch", "elasticsearch"],
  ["neo4j", "neo4j"],
  ["cassandra", "apachecassandra"],
  ["clickhouse", "clickhouse"],
  // Phase 2
  ["jetty", "jetty"],
  ["wildfly", "wildfly"],
  ["ktor", "ktor"],
  ["play framework", "playframework"],
  ["grails", "grails"],
  ["hypercorn", "hypercorn"],
  ["gradio", "gradio"],
  ["fastify", "fastify"],
  ["hapi", "hapi"],
  ["koa", "koajs"],
  ["adonisjs", "adonisjs"],
  ["remix", "remix"],
  ["symfony", "symfony"],
  ["codeigniter", "codeigniter"],
  ["wordpress", "wordpress"],
  ["sinatra", "sinatra"],
  ["erlang", "erlang"],
  ["axum", "axum"],
  ["actix web", "actix"],
  ["rocket", "rocket"],
  ["warp", "warp"],
  ["gin", "gin"],
  ["fiber", "gofiber"],
  ["echo", "echo"],
  ["dart", "dart"],
  ["flutter", "flutter"],
  ["grpc", "grpc"],
  ["jaeger", "jaeger"],
  ["opentelemetry collector", "opentelemetry"],
  ["envoy", "envoy"],
  // Generic runtimes (low confidence)
  ["python", "python"],
  ["ruby", "ruby"],
]);

/** @param {PortItem[]} ports */
export function groupPorts(ports) {
  /** @type {Map<string, PortItem[]>} */
  const groups = new Map();

  for (const port of ports) {
    const key = port.pid === null ? port.id : `pid:${port.pid}`;
    const group = groups.get(key);
    if (group) group.push(port);
    else groups.set(key, [port]);
  }

  return [...groups.values()];
}

/** @param {PortItem} port */
export function brandSlug(port) {
  const framework = port.framework?.name.toLowerCase();
  if (framework && frameworkBrands.has(framework)) return frameworkBrands.get(framework) ?? null;

  const text = [port.processName, port.displayName, port.command, port.executablePath]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();

  return appBrands.find(([terms]) => terms.some((term) => text.includes(term)))?.[1] ?? null;
}

/**
 * Resolve a brand slug to a loadable icon URL (Simple Icons CDN, or a full URL
 * mapping passed through as-is).
 * @param {string | null} brand
 * @returns {string | null}
 */
export function brandIconUrl(brand) {
  if (!brand) return null;
  return brand.startsWith("https://") ? brand : `https://cdn.simpleicons.org/${brand}`;
}

/**
 * Ordered icon-source candidates for a listener, honoring issue #24 precedence:
 * a confidently detected runtime/service/framework brand (Java, OpenSSH,
 * TablePlus, Antigravity, Spring Boot, …) wins over an unrelated project-local
 * favicon. When a brand is present we deliberately do NOT include the local
 * favicon as a fallback — if the brand's remote icon fails, the component falls
 * back to a built-in Lucide category icon, never to the project (or PortPeek)
 * favicon. The local favicon is only used when no brand is detected, i.e. for
 * an otherwise-generic web project.
 * @param {PortItem} port
 * @param {string | null} localSource resolved local-favicon URL, if any
 * @returns {string[]} candidate URLs in priority order
 */
export function iconSources(port, localSource) {
  const brandSource = brandIconUrl(brandSlug(port));
  if (brandSource) return [brandSource];
  return localSource ? [localSource] : [];
}

/**
 * A bound socket is "exposed" when it listens on a non-loopback address
 * (0.0.0.0, ::, or a concrete LAN IP) — reachable from other machines.
 * @param {string} address
 */
export function isExposed(address) {
  if (!address) return false;
  const a = address.toLowerCase();
  return !(a === "127.0.0.1" || a.startsWith("127.") || a === "::1");
}

/**
 * Best-effort provenance from the relay process — not true container mapping.
 * Docker Desktop and WSL forward ports through host-side relay processes.
 * @param {PortItem} port
 * @returns {"docker" | "wsl" | null}
 */
export function portSource(port) {
  const text = [port.processName, port.executablePath, port.command]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
  if (/wslrelay|wslhost|\bwsl\b/.test(text)) return "wsl";
  if (/com\.docker|dockerd|docker desktop|vpnkit|\bdocker\b/.test(text)) return "docker";
  return null;
}

/** @param {PortItem} port */
export function isDatabase(port) {
  const text = [port.processName, port.displayName, port.command, port.executablePath]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
  return ["sqlservr.exe", "postgres", "mysql", "mariadb", "mongo", "redis", "sqlite", "cockroach", "influx", "elastic", "neo4j", "cassandra", "clickhouse"]
    .some((term) => text.includes(term));
}
