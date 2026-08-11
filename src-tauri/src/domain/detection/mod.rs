pub mod favicon;
pub mod framework;
pub mod project;
pub mod types;

use tauri::{AppHandle, Manager};

use crate::domain::ports::types::PortItem;
use crate::infrastructure::cache::{EnrichmentCache, EnrichmentValue};

pub fn enrich(app: &AppHandle, items: &mut [PortItem]) {
    app.state::<EnrichmentCache>().apply(items, |item| {
        let root = project::find_root(item.working_directory.as_deref());
        EnrichmentValue {
            framework: framework::detect(
                item.process_name.as_deref(),
                item.command.as_deref(),
                root.as_deref(),
            ),
            cached_favicon_path: root
                .as_deref()
                .and_then(|root| favicon::cache_project_favicon(app, root)),
        }
    });
}
