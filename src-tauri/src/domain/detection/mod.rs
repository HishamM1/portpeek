pub mod favicon;
pub mod framework;
pub mod project;
pub mod types;

use tauri::AppHandle;

use crate::domain::ports::types::PortItem;

pub fn enrich(app: &AppHandle, items: &mut [PortItem]) {
    for item in items {
        let root = project::find_root(item.working_directory.as_deref());
        item.framework = framework::detect(
            item.process_name.as_deref(),
            item.command.as_deref(),
            root.as_deref(),
        );
        item.cached_favicon_path = root
            .as_deref()
            .and_then(|root| favicon::cache_project_favicon(app, root));
    }
}
