use std::{env::VarError, error::Error};

use wasmtime::{Engine, Store, component::{Component, TypedFunc,Instance, Linker}};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder};

use crate::{VFS_DIR, enums::errors::GlobalError, general, structs::states::WasiState, traits::builder::WasiUtilsBuild};



#[derive(Default)]
pub struct WasiBuild<P,R> 
where 
    P: wasmtime::component::Lower + wasmtime::component::ComponentNamedList,
    R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift
{
    wasi:WasiCtx,
    linker:Option<Linker<WasiState>>,
    component:Option<Component>,
    store:Option<Store<WasiState>>,
    engine:Engine,
    instance:Option<Instance>,
    
    params:P,                              
    returns:R
}


impl<P,R> WasiBuild<P,R> 
where 
    P: wasmtime::component::Lower + wasmtime::component::ComponentNamedList + std::default::Default,
    R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift
{
    pub fn get_store(&mut self)->Result<&mut Store<WasiState>, Box<dyn Error>>
    {
        if let Some(store) = &mut self.store {
            Ok(store)
        } else {
            Err(Box::new(GlobalError::UninitializedVariable))
        }
    }
}


impl<P,R> WasiUtilsBuild<P,R> for WasiBuild<P,R> 
where 
    P: wasmtime::component::Lower + wasmtime::component::ComponentNamedList ,
    R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift
{


    fn set_engine(&mut self) {
        self.engine = Engine::default();
    }

    fn set_linker(&mut self) ->Result<(), Box<dyn Error>>
    {
        let mut link:Linker<WasiState> = Linker::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_sync::<WasiState>(&mut link)?;
        self.linker = Some(link);
        Ok(())
    }

    fn set_store(&mut self)->Result<(), Box<dyn Error>>
    {
        if let Some(vsf_dir) = VFS_DIR.get() {
            let wasi =  WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_network()
            .preopened_dir(vsf_dir, "/fs", wasmtime_wasi::DirPerms::all(), wasmtime_wasi::FilePerms::all())?
            .build();
            self.store = Some(Store::new(&self.engine, WasiState::new(wasi, ResourceTable::new())));
            Ok(())
        } else {
            Err(Box::new(VarError::NotPresent))
        }
        
    }

    fn set_component(&mut self)->Result<(), Box<dyn Error>>
    {
        self.component = Some(Component::from_file(&self.engine, "./target/wasm32-wasip2/release/fs_handler_wasi.wasm")?);
        Ok(())
    }

    fn set_instance(&mut self)->Result<(), Box<dyn Error>>
    {
        if let (Some(linker),Some(store),Some(component)) = (&mut self.linker,&mut self.store,&self.component) {
            self.instance = Some(linker.instantiate(store , component)?);
            Ok(())
        } else {
            Err(Box::new(GlobalError::UninitializedVariable))
        }
    }

    fn build(&mut self,func_name:&str)-> Result<TypedFunc<P,R>, Box<dyn Error>>
    {
        if let (Some(instance),Some(store)) = (self.instance,&mut self.store){
            general::call_wasi_resolve::<P,R>(instance,store, func_name)
        } else {
            Err(Box::new(GlobalError::UninitializedVariable))
        }
    }
}