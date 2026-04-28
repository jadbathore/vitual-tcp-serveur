use std::io;

use commun_utils_handler::fs_strategies::ReadStrategy;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

use crate::{CACHE_CAP,structs::payloads::payload::DataFile};



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



#[derive(Default)]
pub struct PredicatorCache {
    cache_use:u64,

}

impl PredicatorCache {

    pub fn predicate_cache_use(&mut self,type_data:&DataFile)->Result<bool,io::Error>
    {   
        match type_data.get_strategy() {
            ReadStrategy::Smale => {
                let condition = CACHE_CAP > (type_data.size()? + self.cache_use);
                if condition {
                    self.cache_use += type_data.size()?;
                }
                Ok(condition)
            },
            _ => Ok(false)
        }
    }

}