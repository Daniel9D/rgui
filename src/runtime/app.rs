use crate::core::{DisplayList, ResourceStore};

#[derive(Clone, Debug, Default)]
pub struct AppOptions {
    pub title: String,
}

pub struct App {
    options: AppOptions,
    resources: ResourceStore,
}

impl App {
    pub fn new(options: AppOptions) -> Self {
        Self {
            options,
            resources: ResourceStore::default(),
        }
    }

    pub fn build_display_list(&self) -> DisplayList {
        let _ = (&self.options, &self.resources);
        DisplayList::default()
    }
}
