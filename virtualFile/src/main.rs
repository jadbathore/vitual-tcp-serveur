
use std::{env, error::Error, path::PathBuf, sync::{Arc, OnceLock}};
use lazy_static::lazy_static;

mod general;
mod structs;
mod general_macros;
mod traits;


use colored::Colorize;
use commun_utils_handler::{
    errors::GlobalError,
    fs_strategies::recursive_file_read
};
use tokio::{net::TcpListener, runtime::Runtime};
use crate::{general::handle_client, structs::{builder::wasi::build_wasi_call, states::PredicatorCache}};
use crate::{
    structs::{
        iterator::{cached_data::{CacheCollection,StaticCollection},file_info_reader::PayloadCollection}, 
        payloads::payload::DataFile
    }
};




// thread_local! {
//     static BUFFERS: RefCell<Vec<Vec<u8>>> = RefCell::new(panic!("no buffer provided"));

//     // static NEXT_ID: RefCell<u32> = RefCell::new(0);
// }

    static CACHE_CAP:u64 = 1 * 1024 * 1024 * 1024; 

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
        let mut predicator:PredicatorCache = PredicatorCache::default();
        recursive_file_read(path,&mut |file| {
            let datafile = DataFile::new(file)?;
            if predicator.predicate_cache_use(&datafile)? {
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

fn format_message(str:&str)
{
    let size_to_center = 4 + str.len();
    let blankfiller = " ".repeat(size_to_center).on_green();

    println!("\n\t{}",blankfiller);
    println!("\t{:^size_to_center$}",str.white().bold().on_green());
    println!("\t{}",blankfiller);

}

fn main()->Result<(),Box<dyn Error>> 
{
    set_env_var()?;
    build_wasi_call::<(),()>((), "TA0043").map_err(|_|{
        println!("{}",GlobalError::WasiError);
        GlobalError::WasiError
    })?;
    set_payload_variable(VFS_DIR.get())?;
    if let (Some(payloads),Some(caches),Some(addr)) = (PAYLOADS.get(),CACHE_PAYLOADS.get(),ADDRESS.get()) {
        Runtime::new()?.block_on(async {
            let listener = TcpListener::bind(addr).await.unwrap();
            format_message(&("running websocket on ".to_owned() + addr));
            while let Ok((stream, socket_addr)) = listener.accept().await {
                tokio::spawn(handle_client(stream,payloads.iter(),caches.iter()));
                let time = time::OffsetDateTime::now_utc();
                println!("data sended at {time} to {}",socket_addr.to_string().green());
            }
        });
    }   
    Ok(())
}
