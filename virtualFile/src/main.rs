use std::{env, error::Error, path::{PathBuf}, sync::{Arc, OnceLock}};
// use futures::{FutureExt};
use lazy_static::lazy_static;
use tokio::{net::TcpListener, runtime::Runtime};

// use fs_handler_wasi::structs::{collection::{self, PayloadCollection}, json_struct::JsonInfo, payload_request::{self, DataFile}};
// use futures::StreamExt;
mod general;
mod structs;
mod general_macros;
mod traits;

use fs_handler_wasi::commun_utils::{
    read_strategies::{ReadStrategy,recursive_file_read},
    error::GlobalError
};

use crate::structs::{builder::wasi::build_wasi_call, iterator::cached_data::StaticCollection};
use crate::{
    general::handle_client, 
    structs::{
        builder::{director::Director, wasi::WasiBuild}, 
        iterator::{cached_data::CacheCollection,file_info_reader::PayloadCollection}, 
        payloads::payload::DataFile
    }, 
    traits::builder::WasiUtilsBuild
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

fn set_payload_variable(vfs_path:Option<&PathBuf>)->Result<(), Box<GlobalError>>
{
    if let Some(path) = vfs_path {
        let mut data_to_payload:Vec<DataFile> = Vec::new();
        let mut data_to_cache:Vec<DataFile> = Vec::new();
        recursive_file_read(path,&mut |file| {
            let datafile = DataFile::new(file)?;
            if predicate_cache_use(&datafile) {
                data_to_cache.push(datafile);
            } else {
                data_to_payload.push(datafile);
            }
            Ok(())
        }).map_err(|_|{Box::new(GlobalError::NotExistingDir(path.to_string_lossy().to_string()))})?;
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
    let a = build_wasi_call::<(),()>((), "TA0043")?;

    // set_payload_variable(VFS_DIR.get())?;

    // if let (Some(payloads),Some(caches),Some(addr)) = (PAYLOADS.get(),CACHE_PAYLOADS.get(),ADDRESS.get()) {
    //     Runtime::new()?.block_on(async {
    //         let listener = TcpListener::bind(addr).await.unwrap();
    //         println!("running websocket on {}",addr);
    //         while let Ok((stream, addr)) = listener.accept().await {
    //             tokio::spawn(handle_client(stream,payloads.iter(),caches.iter()));
    //             let time = time::OffsetDateTime::now_utc();
    //             println!("data sended at {time} to {addr}");
    //         }
    //     });
    // }   
    Ok(())
}
