use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

// Host State containing capabilities, resources, and UI handles
pub struct HostState {
    pub ctx: WasiCtx,
    pub http_ctx: WasiHttpCtx,
    pub table: ResourceTable,

    // Add capabilities allowed for this execution
    pub allowed_capabilities: Vec<String>,

    // Meta-information for rich diagnostics
    pub _manifest_path: Option<String>,
    pub _manifest_content: Option<String>,
}

impl HostState {
    pub fn new(
        allowed_capabilities: Vec<String>,
        manifest_path: Option<String>,
        manifest_content: Option<String>,
    ) -> Self {
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()
            .inherit_env() // Allow plugin to see PATH and other env vars
            .build();

        let http_ctx = WasiHttpCtx::new();

        Self {
            ctx,
            http_ctx,
            table: ResourceTable::new(),
            allowed_capabilities,
            _manifest_path: manifest_path,
            _manifest_content: manifest_content,
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

impl WasiHttpView for HostState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}
