use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};



pub struct WasiState {
    ctx:WasiCtx,
    table:ResourceTable
}


impl WasiState {
    pub fn new(wasi_ctx:WasiCtx,table:ResourceTable)->Self 
    {
        WasiState { 
            ctx: wasi_ctx, 
            table: table 
        }
    }
}

impl WasiView for WasiState {

    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        WasiCtxView { ctx: &mut self.ctx, table: &mut self.table }
    }
}