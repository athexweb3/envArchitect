// internal/mod.rs

// 1. Generate bindings in a sub-module to allow SDK helpers (api::host) to call Host functions.
pub mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/plugin.wit",
        world: "plugin",
    });
}
