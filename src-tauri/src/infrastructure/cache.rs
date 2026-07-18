use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use crate::domain::{detection::framework::FrameworkDetection, ports::types::PortItem};

// ponytail: retain() already bounds the map to the current listener count
// (dozens in practice), so this cap is only a backstop against a single scan
// flooding the map. On overflow we clear wholesale — a cliff, but unreachable
// in normal use. Swap for an LRU crate only if that ever stops being true.
const MAX_ENTRIES: usize = 512;

/// Stable identity of an enriched listener. Covers every input the enrichment
/// pipeline reads — process name and command (framework detection) and working
/// directory (project root → framework + favicon) — plus the PID and executable
/// so a port that changes owner, or a reused PID, misses. Because the key is the
/// full input set, a cache hit is guaranteed to equal a fresh compute, and any
/// input change (including PID reuse with a different command) misses.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EnrichmentKey {
    pid: Option<u32>,
    process_name: Option<String>,
    command: Option<String>,
    executable_path: Option<String>,
    working_directory: Option<String>,
}

impl EnrichmentKey {
    fn of(item: &PortItem) -> Self {
        Self {
            pid: item.pid,
            process_name: item.process_name.clone(),
            command: item.command.clone(),
            executable_path: item.executable_path.clone(),
            working_directory: item.working_directory.clone(),
        }
    }
}

/// The enrichment fields that are expensive to recompute (filesystem work).
///
/// ponytail: cached for the whole lifetime of a listener, including `None`
/// results and the favicon path. That is the point of issue #26 — a stable
/// process is enriched once, not every 2s. The tradeoff: a favicon or config
/// added/changed while the process keeps running is not picked up until the
/// process restarts. Add an mtime/source check here if live refresh matters.
#[derive(Clone)]
pub struct EnrichmentValue {
    pub framework: Option<FrameworkDetection>,
    pub cached_favicon_path: Option<String>,
}

#[derive(Default)]
pub struct EnrichmentCache {
    entries: Mutex<HashMap<EnrichmentKey, EnrichmentValue>>,
}

impl EnrichmentCache {
    /// Applies cached enrichment to `items`, computing (and storing) only for
    /// listeners not already cached. Entries for listeners absent from this
    /// scan are evicted, so the map tracks the live set.
    pub fn apply<F>(&self, items: &mut [PortItem], mut compute: F)
    where
        F: FnMut(&PortItem) -> EnrichmentValue,
    {
        let keys: Vec<EnrichmentKey> = items.iter().map(EnrichmentKey::of).collect();

        // Phase 1: serve hits under the lock, note misses. Lock released before
        // any filesystem work so concurrent refreshes don't serialize on disk.
        let mut values: Vec<Option<EnrichmentValue>> = {
            let map = self.entries.lock().expect("enrichment cache poisoned");
            keys.iter().map(|key| map.get(key).cloned()).collect()
        };

        // Phase 2: compute misses without holding the lock, deduplicating by
        // key so a process with several listeners (dual-stack / multiple ports)
        // is enriched once, not once per listener.
        let mut fresh: HashMap<EnrichmentKey, EnrichmentValue> = HashMap::new();
        for (index, slot) in values.iter_mut().enumerate() {
            if slot.is_none() {
                let value = match fresh.get(&keys[index]) {
                    Some(value) => value.clone(),
                    None => {
                        let value = compute(&items[index]);
                        fresh.insert(keys[index].clone(), value.clone());
                        value
                    }
                };
                *slot = Some(value);
            }
        }

        // Phase 3: evict listeners absent from this scan (vanished processes and
        // reused PIDs), THEN store freshly computed values. Evicting first keeps
        // the map from transiently growing to old+new entries, which could trip
        // the MAX_ENTRIES backstop and clear the whole cache spuriously.
        {
            let mut map = self.entries.lock().expect("enrichment cache poisoned");
            let live: HashSet<&EnrichmentKey> = keys.iter().collect();
            map.retain(|key, _| live.contains(key));
            for (key, value) in fresh {
                if map.len() >= MAX_ENTRIES {
                    map.clear();
                }
                map.insert(key, value);
            }
        }

        for (item, value) in items.iter_mut().zip(values) {
            let value = value.expect("every listener resolved to a value");
            item.framework = value.framework;
            item.cached_favicon_path = value.cached_favicon_path;
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listener(pid: u32, cwd: &str) -> PortItem {
        use crate::domain::ports::types::PortProtocol;
        PortItem {
            id: format!("{pid}-{cwd}"),
            port: 3000,
            address: "127.0.0.1".into(),
            protocol: PortProtocol::Tcp,
            pid: Some(pid),
            process_name: Some("node".into()),
            display_name: None,
            memory_mb: None,
            uptime_seconds: None,
            command: None,
            executable_path: Some(format!("C:/{cwd}/node.exe")),
            working_directory: Some(cwd.into()),
            url: None,
            favicon_url: None,
            cached_favicon_path: None,
            framework: None,
            is_system_port: false,
        }
    }

    fn value(name: &str) -> EnrichmentValue {
        EnrichmentValue {
            framework: None,
            cached_favicon_path: Some(name.into()),
        }
    }

    #[test]
    fn unchanged_listener_is_not_recomputed() {
        let cache = EnrichmentCache::default();
        let mut calls = 0;
        let mut compute = |_: &PortItem| {
            calls += 1;
            value("vite")
        };

        let mut scan1 = vec![listener(100, "app")];
        cache.apply(&mut scan1, &mut compute);
        let mut scan2 = vec![listener(100, "app")];
        cache.apply(&mut scan2, &mut compute);

        assert_eq!(calls, 1, "second scan must reuse the cached value");
        assert_eq!(scan2[0].cached_favicon_path.as_deref(), Some("vite"));
    }

    #[test]
    fn new_listener_is_computed_once_then_served() {
        let cache = EnrichmentCache::default();
        let mut calls = 0;
        let mut compute = |_: &PortItem| {
            calls += 1;
            value("next")
        };

        let mut first = vec![listener(1, "a")];
        cache.apply(&mut first, &mut compute);
        let mut second = vec![listener(1, "a"), listener(2, "b")];
        cache.apply(&mut second, &mut compute);

        assert_eq!(calls, 2, "only the newly appeared listener is computed");
    }

    #[test]
    fn process_with_multiple_listeners_computes_once() {
        let cache = EnrichmentCache::default();
        let mut calls = 0;
        let mut compute = |_: &PortItem| {
            calls += 1;
            value("astro")
        };

        // Same PID/exe/cwd on two ports (e.g. dual-stack) => one shared key.
        let mut scan = vec![listener(42, "app"), listener(42, "app")];
        cache.apply(&mut scan, &mut compute);

        assert_eq!(
            calls, 1,
            "one process must be enriched once, not per listener"
        );
        assert_eq!(scan[1].cached_favicon_path.as_deref(), Some("astro"));
    }

    #[test]
    fn vanished_and_pid_reuse_entries_are_evicted() {
        let cache = EnrichmentCache::default();
        let mut compute = |item: &PortItem| value(item.working_directory.as_deref().unwrap());

        let mut scan1 = vec![listener(100, "old")];
        cache.apply(&mut scan1, &mut compute);
        assert_eq!(cache.len(), 1);

        // Same PID, different working directory (PID reuse) plus the old entry
        // is gone from this scan: only the current key survives.
        let mut scan2 = vec![listener(100, "new")];
        cache.apply(&mut scan2, &mut compute);
        assert_eq!(cache.len(), 1);
        assert_eq!(scan2[0].cached_favicon_path.as_deref(), Some("new"));
    }

    #[test]
    fn cache_stays_bounded() {
        let cache = EnrichmentCache::default();
        let mut compute = |item: &PortItem| value(item.working_directory.as_deref().unwrap());

        // Each scan replaces the previous live set, so the map never exceeds
        // the per-scan listener count regardless of session length.
        for round in 0..(MAX_ENTRIES * 2) {
            let mut scan = vec![listener(round as u32, &format!("dir{round}"))];
            cache.apply(&mut scan, &mut compute);
        }
        assert!(cache.len() <= MAX_ENTRIES);
    }
}
