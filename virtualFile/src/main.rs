use std::{env, error::Error, path::{PathBuf}, sync::{Arc, OnceLock}};
// use futures::{FutureExt};
use lazy_static::lazy_static;
use tokio::{net::TcpListener, runtime::Runtime};

// use fs_handler_wasi::structs::{collection::{self, PayloadCollection}, json_struct::JsonInfo, payload_request::{self, DataFile}};
// use futures::StreamExt;
mod general;
mod structs;
mod enums;
mod general_macros;
mod traits;

use crate::structs::iterator::cached_data::StaticCollection;
use crate::{
    enums::errors::GlobalError, general::handle_client, structs::{
        builder::{director::Director, wasi::WasiBuild}, iterator::{cached_data::CacheCollection,file_info_reader::PayloadCollection}, payloads::payload::{self, DataFile}, read_strategies::ReadStrategy, resolver::recusive_dispacher
    }, traits::builder::WasiUtilsBuild
};

// thread_local! {
//     static BUFFERS: RefCell<Vec<Vec<u8>>> = RefCell::new(panic!("no buffer provided"));

//     // static NEXT_ID: RefCell<u32> = RefCell::new(0);
// }


lazy_static!(
    static ref VFS_DIR:OnceLock<PathBuf> = OnceLock::new();
    static ref ADDRESS:OnceLock<String> = OnceLock::new();
    static ref PROTOCOLS:[&'static str;2] = ["write","read"];
    static ref CACHE_PAYLOADS:OnceLock<Arc<CacheCollection>> = OnceLock::new();
    static ref PAYLOADS:OnceLock<Arc<PayloadCollection>> =OnceLock::new();
);

fn build_wasi_call<P,R>(param:P,func_name:&str)->Result<R, Box<dyn Error>>
where 
    P: wasmtime::component::Lower + wasmtime::component::ComponentNamedList + std::default::Default,
    R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift + std::default::Default,
{
    let mut builder:WasiBuild<P, R> = WasiBuild::default();
    Director::construct_wasi(&mut builder)?;
    let typed_req = builder.build(func_name)?;
    let a = typed_req.call(builder.get_store()?,param)?;
    Ok(a)
}

fn error_handle_set_oncelock<T>(_:T)->Box<GlobalError>
{
    Box::new(GlobalError::ResetOnceLock)
}

fn predicate_cache_use(type_data:&DataFile)->bool
{
    match type_data.get_strategy() {
        ReadStrategy::Smale => true,
        _ => false
    }
}

fn set_env_var()->Result<(), Box<dyn Error>>
{

    if let (Ok(vfs_path),Ok(address)) = (env::var("VFS_DIR"),env::var("ADDRESS")) {
        let path = PathBuf::from(vfs_path);
        VFS_DIR.set(path).map_err(error_handle_set_oncelock)?;
        ADDRESS.set(address).map_err(error_handle_set_oncelock)?;
        Ok(())
    } else {
        Err(Box::new(GlobalError::UninitializedVariable))
    }
}

fn set_payload_variable<'a>(vfs_path:Option<&PathBuf>)->Result<(), Box<GlobalError>>
{
    if let Some(path) = vfs_path {
        let mut data_to_payload:Vec<DataFile> = Vec::new();
        let mut data_to_cache:Vec<DataFile> = Vec::new();
        recusive_dispacher(path, &mut data_to_payload,&mut data_to_cache).map_err(|_|{Box::new(GlobalError::NotExistingDir(path.to_string_lossy().to_string()))})?;
        let payload = PayloadCollection::from(data_to_payload);
        let cache = CacheCollection::try_from(data_to_cache)?;
        PAYLOADS.set(Arc::from(PayloadCollection::from(payload))).map_err(error_handle_set_oncelock)?;
        CACHE_PAYLOADS.set(Arc::from(CacheCollection::from(cache))).map_err(error_handle_set_oncelock)?;
    }
    Ok(())
}

fn main()->Result<(),Box<dyn Error>> 
{
    set_env_var()?;

    set_payload_variable(VFS_DIR.get())?;

    if let (Some(payloads),Some(caches),Some(addr)) = (PAYLOADS.get(),CACHE_PAYLOADS.get(),ADDRESS.get()) {
        Runtime::new()?.block_on(async {
            let listener = TcpListener::bind(addr).await.unwrap();
            println!("running websocket on {}",addr);
            while let Ok((stream, addr)) = listener.accept().await {
                tokio::spawn(handle_client(stream,payloads.iter(),caches.iter()));
                let time = time::OffsetDateTime::now_utc();
                println!("data sended at {time} to {addr}");
            }
        });
    }   
    Ok(())
}
