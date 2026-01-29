use std::error::Error;

use crate::traits::builder::WasiUtilsBuild;



pub struct Director;

impl Director {
    pub fn construct_wasi<P,R>(builder: &mut impl WasiUtilsBuild<P,R>)->Result<(),Box<dyn Error>>
    where 
        P: wasmtime::component::Lower + wasmtime::component::ComponentNamedList,
        R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift
    {
        builder.set_engine();
        builder.set_linker()?;
        builder.set_store()?;
        builder.set_component()?;
        builder.set_instance()?;
        Ok(())
    }
}
