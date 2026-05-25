
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};


#[cfg(feature = "client")]
use crate::CACHE_CAP;

// #[cfg(feature = "client")]
// use std::{error::Error, io, path::Path};

#[cfg(feature = "client")]
use commun_utils_handler::fs_strategies::MEDIUM_FILE;

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


#[cfg(feature = "client")]
#[derive(Default)]
pub struct PredicatorCache {
    cache_use:u64,

}


#[cfg(feature = "client")]
impl PredicatorCache {
    pub const fn predicate_cache_use(&mut self,size:u64)->bool
    {   
        if size <= MEDIUM_FILE {
            let condition = CACHE_CAP > (size + self.cache_use);
            if condition {
                self.cache_use += size;
            }
            return condition;
        } 
        false
    }
}