pub mod state;
pub mod ui;

// Wrap bindings in a public module to keep the namespace clean
// and to allow `use super::bindings::...` in sibling modules.
pub mod bindings {
    use wasmtime::component::bindgen;
    bindgen!({
        path: "../../packages/sdks/wit/plugin.wit",
        world: "plugin",
        async: true,
    });
}
