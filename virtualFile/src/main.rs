use std::{env, error::Error, path::PathBuf, sync::OnceLock};

use lazy_static::lazy_static;

mod general;
mod structs;
mod general_macros;
mod traits;
mod runtime;
use colored::Colorize;
use commun_utils_handler::errors::GlobalError;
use tokio::{net::TcpListener, runtime::Runtime};

#[cfg(feature = "client")]
use {
    crate::{
        general::{handle_client,client_resolve_directories},
        structs::{
            iterator::collections::{CacheCollection,PayloadCollection,StaticAssetsCollection},
            payloads::payload::DataFile,
            states::PredicatorCache
        },
    },
    std::sync::Arc
};

#[cfg(feature = "deamon")]
use crate::general::handle_deamon;

#[cfg(feature = "client")]
use crate::{runtime::FakeToSubPath, structs::{async_strategies::FileAsyncReader}};
// use crate::structs::{builder::wasi::build_wasi_call};


#[cfg(feature = "client")]
use commun_utils_handler::fs_strategies::FileReader;

lazy_static!(
    static ref VFS_DIR:OnceLock<PathBuf> = OnceLock::new();
    static ref ADDRESS:OnceLock<String> = OnceLock::new();
);


#[cfg(feature = "client")]
lazy_static!(
    static ref CACHES:OnceLock<Arc<CacheCollection>> = OnceLock::new();
    static ref PAYLOADS:OnceLock<Arc<PayloadCollection>> = OnceLock::new();
);

#[cfg(feature = "client")]
static ASSETS:OnceLock<Arc<StaticAssetsCollection>> = OnceLock::new();

#[cfg(feature = "client")]
static CACHE_CAP:u64 = 1 * 1024 * 1024 * 1024; 


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

#[cfg(feature = "client")]
fn set_payload_variable(vfs_path:Option<&PathBuf>)->Result<(), Box<GlobalError>>
{
    if let Some(path) = vfs_path {
        let mut data_to_payload:Vec<DataFile<FileAsyncReader<FakeToSubPath>>> = Vec::new();
        let mut data_to_cache:Vec<DataFile<FileReader<FakeToSubPath>>> = Vec::new();
        let mut predicator:PredicatorCache = PredicatorCache::default();
        client_resolve_directories(path,&mut |fake_path|{
            if predicator.predicate_cache_use(fake_path.metadata()?.len()) { 
                data_to_cache.push(DataFile::new(fake_path.try_into()?)?);
            } else {
                data_to_payload.push(DataFile::new(fake_path.to_path_buf().try_into()?)?);
            }
            Ok(())
        }).map_err(|_|{Box::new(GlobalError::NotExistingDir(path.to_string_lossy().to_string()))})?;
        let payload:PayloadCollection = PayloadCollection::try_from(data_to_payload)?;
        let cache:CacheCollection = CacheCollection::try_from(data_to_cache)?;
        PAYLOADS.set(Arc::from(payload)).map_err(error_handle_set_oncelock)?;
        CACHES.set(Arc::from(cache)).map_err(error_handle_set_oncelock)?;
        ASSETS.set(Arc::from(StaticAssetsCollection::new()?)).map_err(error_handle_set_oncelock)?;
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

    // build_wasi_call::<(),()>((), "TA0043").map_err(|_|{
    //     println!("{}",GlobalError::WasiError);
    //     GlobalError::WasiError
    // })?;

    #[cfg(feature = "client")]
    {
        set_payload_variable(VFS_DIR.get())?;
        if let (Some(assets),Some(addr)) = (ASSETS.get(),ADDRESS.get()) {
            Runtime::new()?.block_on(async {
                let listener = TcpListener::bind(addr).await.unwrap();
                format_message(&("client-websocket on ".to_owned() + addr));
                while let Ok((stream, socket_addr)) = listener.accept().await {
                    tokio::spawn(handle_client(stream,assets));
                    let time = time::OffsetDateTime::now_utc();
                    println!("data sended at {time} to {}",socket_addr.to_string().green());
                }
            });
        }  
    }

    #[cfg(feature = "deamon")]
    {
        if let Some(addr) = ADDRESS.get() {
            Runtime::new()?.block_on(async {
                let listener = TcpListener::bind(addr).await.unwrap();
                format_message(&("deamon-websocket on ".to_owned() + addr));
                while let Ok((stream, socket_addr)) = listener.accept().await {
                    tokio::spawn(handle_deamon(stream));
                    let time = time::OffsetDateTime::now_utc();
                    println!("data sended at {time} to {}",socket_addr.to_string().green());
                }
            });
        } 
    }
    Ok(())
}