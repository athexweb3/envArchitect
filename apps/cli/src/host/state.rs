use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

// Host State containing capabilities, resources, and UI handles
pub struct HostState {
    pub ctx: WasiCtx,
    pub table: ResourceTable,

    // Add capabilities allowed for this execution
    pub allowed_capabilities: Vec<String>,

    // Meta-information for rich diagnostics
    pub manifest_path: Option<String>,
    pub manifest_content: Option<String>,
}

impl HostState {
    pub fn new(
        allowed_capabilities: Vec<String>,
        manifest_path: Option<String>,
        manifest_content: Option<String>,
    ) -> Self {
        let ctx = WasiCtxBuilder::new().inherit_stdio().inherit_args().build();

        Self {
            ctx,
            table: ResourceTable::new(),
            allowed_capabilities,
            manifest_path,
            manifest_content,
        }
    }
}

impl WasiView for HostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}
