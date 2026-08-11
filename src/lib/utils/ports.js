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
  ["python", "python"],
  ["ruby", "ruby"],
]);

/** @param {PortItem[]} ports @param {number[]} [pinnedPorts] */
export function groupPorts(ports, pinnedPorts = []) {
  /** @type {Map<string, PortItem[]>} */
  const groups = new Map();

  for (const port of ports) {
    const key = port.pid === null ? port.id : `pid:${port.pid}`;
    const group = groups.get(key);
    if (group) group.push(port);
    else groups.set(key, [port]);
  }

  const pinned = new Set(pinnedPorts);
  return [...groups.values()].sort(
    (a, b) => Number(b.some((item) => pinned.has(item.port))) - Number(a.some((item) => pinned.has(item.port))),
  );
}

/** @param {PortItem[]} ports @param {boolean} showSystemPorts @param {boolean} showUdp @param {string} query @param {number[]} [pinnedPorts] */
export function filterPorts(ports, showSystemPorts, showUdp, query, pinnedPorts = []) {
  const needle = query.trim().toLowerCase();
  return ports.filter((port) => {
    if (!needle) {
      if (pinnedPorts.includes(port.port)) return true;
      if (!showSystemPorts && port.isSystemPort) return false;
      if (!showUdp && port.protocol === "udp") return false;
      return true;
    }
    return [
      port.port.toString(),
      port.address,
      port.processName,
      port.displayName,
      port.framework?.name,
      port.pid?.toString(),
    ].some((value) => value?.toLowerCase().includes(needle));
  });
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

/** @param {string | null} brand @returns {string | null} */
export function brandIconUrl(brand) {
  if (!brand) return null;
  return brand.startsWith("https://") ? brand : `https://cdn.simpleicons.org/${brand}`;
}

/** @param {PortItem} port @param {string | null} localSource @returns {string[]} */
export function iconSources(port, localSource) {
  const brandSource = brandIconUrl(brandSlug(port));
  if (brandSource) return [brandSource];
  return localSource ? [localSource] : [];
}

/** @param {string} address */
export function isExposed(address) {
  if (!address) return false;
  const a = address.toLowerCase();
  return !(a === "127.0.0.1" || a.startsWith("127.") || a === "::1");
}

/** @param {PortItem} port @returns {"docker" | "wsl" | null} */
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
