use std::error::Error;

use wasmtime::component::TypedFunc;

pub trait WasiUtilsBuild<P,R>
    where 
        P: wasmtime::component::Lower + wasmtime::component::ComponentNamedList,
        R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift
{

    // type Param: wasmtime::component::Lower + wasmtime::component::ComponentNamedList;
    // type Return: wasmtime::component::ComponentNamedList + wasmtime::component::Lift;
    
    fn set_engine(&mut self);
    fn set_linker(&mut self)->Result<(), Box<dyn Error>>;
    fn set_store(&mut self)->Result<(), Box<dyn Error>>;
    fn set_component(&mut self)->Result<(), Box<dyn Error>>;
    fn set_instance(&mut self)->Result<(), Box<dyn Error>>;
    fn build(&mut self,func_name:&str)->Result<TypedFunc<P,R>, Box<dyn Error>>;
}





// impl<P,R> dyn WasiUtilsBuild<OutputType = TypedFunc<P,R>> 
// where 
//         P: wasmtime::component::Lower + wasmtime::component::ComponentNamedList,
//         R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift
// {
//     fn set_typed_callback(instance:Instance,store:&mut Store<WasiState>,func_name:&str)->Result<TypedFunc<P,R>,Box<dyn Error>>

//     {
//         if let Some(func) = instance.get_func(&mut *store, func_name){
//             let typed:TypedFunc<P,R> = func.typed(&store)?;
//             Ok(typed)
//         } else {
//             Err(Box::new(GlobalError::ParseError(String::from("unknown function s"))))
//         }
//     }
// }