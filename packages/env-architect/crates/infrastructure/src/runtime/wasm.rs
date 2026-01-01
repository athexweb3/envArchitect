use anyhow::{Context, Result};
use std::path::PathBuf;
use wasmtime::{
    Config, Engine, InstanceAllocationStrategy, Linker, Module, PoolingAllocationConfig,
    ResourceLimiter, Store, StoreLimits, StoreLimitsBuilder,
};
use wasmtime_wasi::p1::WasiP1Ctx;
use wasmtime_wasi::WasiCtxBuilder;

/// The secure execution environment for plugins.
/// Features: Pooling Allocator, Epoch Interrupts, Fuel Metering, and Cap-Std FS Isolation.
pub struct PluginRuntime {
    engine: Engine,
}

/// The state managed by the Wasmtime Store.
/// Provides access to the WASI context and resource limits.
struct HostState {
    wasi_ctx: WasiP1Ctx,
    limits: StoreLimits,
}

impl ResourceLimiter for HostState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool> {
        // SECURITY: Strict 64MB Memory Cap (Fallback if pooling isn't used)
        if desired > (64 * 1024 * 1024) {
            return Ok(false);
        }
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool> {
        // SECURITY: Limit table elements to prevent DoS via exhaustion
        if desired > 10000 {
            return Ok(false);
        }
        Ok(true)
    }

    fn instances(&self) -> usize {
        self.limits.instances()
    }

    fn tables(&self) -> usize {
        self.limits.tables()
    }

    fn memories(&self) -> usize {
        self.limits.memories()
    }
}

impl PluginRuntime {
    /// Initialize the runtime with "Military Grade" security settings.
    pub fn new() -> Result<Self> {
        let mut config = Config::new();

        // SECURITY: Use Pooling Allocator for deterministic memory isolation.
        // This pre-allocates memory slots, mitigating heap-spray and fragmentation attacks.
        let mut pooling_config = PoolingAllocationConfig::default();
        // Limit each instance to 64MB linear memory.
        pooling_config.total_memories(100); // Allow up to 100 concurrent plugins
        pooling_config.max_memory_size(64 * 1024 * 1024);

        config.allocation_strategy(InstanceAllocationStrategy::Pooling(pooling_config));

        // SECURITY: Enable Epoch Interruption (Zero-overhead async killing)
        config.epoch_interruption(true);

        // SECURITY: Enable Fuel Consumption (Instruction-level limiting)
        config.consume_fuel(true);

        let engine = Engine::new(&config).context("Failed to initialize Wasmtime engine")?;

        Ok(Self { engine })
    }

    /// Execute a plugin binary with strict sandbox constraints.
    pub fn run(&self, wasm_bytes: &[u8], allowed_paths: Vec<PathBuf>) -> Result<()> {
        let mut linker = Linker::new(&self.engine);

        // Map WASI Preview 1 imports to our host state
        wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |s: &mut HostState| &mut s.wasi_ctx)?;

        // Build WASI Context (Sandboxed Env)
        let mut wasi_builder = WasiCtxBuilder::new();
        wasi_builder.inherit_stdout().inherit_stderr();

        // SECURITY: Capability-based File System Access
        for path in allowed_paths {
            if path.exists() {
                let name = path.to_string_lossy().into_owned();
                wasi_builder.preopened_dir(
                    &path,
                    name,
                    wasmtime_wasi::DirPerms::all(),
                    wasmtime_wasi::FilePerms::all(),
                )?;
            }
        }

        let wasi_ctx = wasi_builder.build_p1();

        // SECURITY: Define Resource Limits
        let limits = StoreLimitsBuilder::new()
            .memory_size(64 * 1024 * 1024)
            .instances(1)
            .memories(1)
            .tables(1)
            .build();

        let state = HostState { wasi_ctx, limits };

        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| s);

        // SECURITY: Set Epoch Deadline (e.g., 1 refresh to trigger interrupt)
        store.set_epoch_deadline(1);

        // SECURITY: Add Fuel (limit execution steps)
        store
            .set_fuel(1_000_000)
            .context("Failed to set fuel limit")?;

        let module =
            Module::new(&self.engine, wasm_bytes).context("Failed to compile WASM module")?;
        linker.module(&mut store, "", &module)?;

        let instance = linker.instantiate(&mut store, &module)?;
        let start = instance.get_typed_func::<(), ()>(&mut store, "_start")?;

        start
            .call(&mut store, ())
            .context("Plugin execution failed: Out of fuel, memory, or logic error")?;

        Ok(())
    }
}
