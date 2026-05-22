
// #[cfg(feature = "deamon")]
// use std::path::PathBuf;
// #[cfg(feature = "client")]
// use {
//     std::sync::Arc,
//     futures::stream::SplitSink,
// };

use std::{pin::Pin, sync::OnceLock, vec};

#[cfg(feature = "deamon")]
use futures::{StreamExt, stream::SplitStream};
// #[cfg(feature = "deamon")]
// use regex::Regex;

#[cfg(feature = "deamon")]
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use derive_utils::IterableStringifyEnum;

use commun_utils_handler::IterableStringifyEnum;
// use std::path::Path;


use tokio_tungstenite::{WebSocketStream, accept_hdr_async, tungstenite::{ 
        handshake::{client::Request, server::ErrorResponse}, http::{HeaderValue, Response}
    }
};


pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// #[cfg(feature = "deamon")]
// use crate::structs::storage::storage_file;

#[cfg(feature = "client")]
use crate::runtime::FakePath;
#[cfg(feature = "deamon")]
use crate::structs::storage::storage_strategy;

#[cfg(feature = "client")]
use {
    crate::structs::iterator::{
        collections::StaticAssetsCollection,
    },
    {
        tokio_tungstenite::tungstenite::Message,
        std::{sync::Arc,path::Path,error::Error,ffi::OsString},
        commun_utils_handler::fs_strategies::get_entries,
        futures::{stream::SplitSink,StreamExt, stream::SplitStream}
    }
};

#[cfg(feature = "deamon")]
use {
    crate::VFS_DIR,
    std::{path::PathBuf,str::FromStr},
    regex::{Regex,RegexSet},
    tokio_tungstenite::tungstenite::{Error as TungError},
    // zstd::bulk,
};


//---------------------------------------------------------------------------
//----------------------------  general enum  -------------------------------
//---------------------------------------------------------------------------

#[cfg(feature = "client")]
#[derive(IterableStringifyEnum)]
pub enum Protocols {
    Read,
    Write,
    ExecJS,
}


#[cfg(feature = "deamon")]
#[derive(IterableStringifyEnum,Debug)]
pub enum CommandProtocols {
    AddFile,
    AddExecutable,
}


#[cfg(feature = "deamon")]
#[derive(IterableStringifyEnum)]
pub enum Flag {
    File,
    Directory
}

#[cfg(feature = "client")]
pub enum Asset {
    Cache(usize),
    Payload(usize)
}

#[cfg(feature = "client")]
pub type WriteSender = SplitSink<WebSocketStream<TcpStream>,Message>;

pub type ReadSender = SplitStream<WebSocketStream<TcpStream>>;

//---------------------------------------------------------------------------
//----------------------------  Strategies ----------------------------------
//---------------------------------------------------------------------------

// #[cfg(feature = "client")]
pub struct NavigatorProtocols<P:IterableStringifyEnum>
{
    protocols:OnceLock<P>
}
// #[cfg(feature = "client")]
impl<P:IterableStringifyEnum> NavigatorProtocols<P> {

    pub fn new()-> Self 
    {
        NavigatorProtocols { protocols: OnceLock::new() }
    }

    pub fn hands_shake_callback(&self)-> impl FnOnce(&Request,Response<()>)->Result<Response<()>,ErrorResponse> 
    {
        move |req:&Request,mut res:Response<()>| {
                if let Some(p) = req.headers().get("sec-websocket-protocol") {
                    let inner = p.to_str().map_err(|err|{
                        ErrorResponse::new(Some(err.to_string()))
                    })?;
                    if let Ok(protocols) = P::from_str(inner) {
                        let header = inner.parse::<HeaderValue>().map_err(|_|{
                            ErrorResponse::new(Some(String::from("can't parse : ") + inner))
                        })?;
                        res.headers_mut().insert("Sec-WebSocket-Protocol",header);
                        self.protocols.set(protocols).map_err(error_handle_set_oncelock)?;
                    } else {
                        return Err(ErrorResponse::new(Some(String::from("protocole :") + inner + "not accept" )));
                    }
                }
            Ok(res)
        }
    }
}

#[cfg(feature = "deamon")]
impl NavigatorProtocols<CommandProtocols>
{
    pub async fn resolve_protocol(&self,read:&mut ReadSender)->Result<(),TungError>
    {
        if let Some(protocol) = self.protocols.get() {
            match protocol {
                CommandProtocols::AddFile => {
                    if let (Some(Ok(path_name)),Some(Ok(size))) = (read.next().await,read.next().await) {

                        let mut sub_new_file:String = path_name.into_text()?;
                        let regexes = [r"\.+\/",r"\/"," "];
                        let regex_set = RegexSet::new(regexes).map_err(|_| TungError::Utf8)?;
                        for index in regex_set.matches(&sub_new_file).iter() {
                            let replacement = match index {
                                0 => "",
                                1 => "-",
                                _ => ""
                            };
                            let regex = Regex::new(regexes[index]).unwrap();
                            sub_new_file = regex.replace_all(&sub_new_file, replacement).to_string();
                        }
                        
                        let predicate_size = usize::from_str(&size.into_text()?).map_err(|_|TungError::Utf8)?;

                        if let Some(vfs ) = VFS_DIR.get() {
                            let mut path_file:PathBuf = PathBuf::from(vfs);
                            path_file.extend(&PathBuf::from(sub_new_file));
                            // let path = path_file.as_path();
                            // tokio::spawn(storage_strategy(path_file.as_path(), predicate_size));
                            // let mut  a = storage_strategy(path_file.as_path(), predicate_size).await.map_err(|_|TungError::Url(UrlError::UnsupportedUrlScheme)).await??;
                            let mut a = storage_strategy(&path_file, predicate_size).await?;
                            while let Some(Ok(data)) = read.next().await {
                                let _ = a.write(data.into_data().as_slice()).await;
                            }
                            
                        // dbg!(path_file);
                            // let buffer = Vec::new();
                            // while let Some(Ok(chunck)) = read.next().await {
                            //     let data_chunck = chunck.into_data();
                            // }
                            // storage_gestion(&path_file, buffers);
                        } 
                        // let a = path_name.into_text()?.as_str();
                        

                        // while let Some(Ok(chunck)) = read.next().await {
                        //     let data_chunck = chunck.into_data();
                        //     chunck_count += 0;
                        // }
                    }

                    // storage_gestion(path, buffers)
                },
                CommandProtocols::AddExecutable => {
                    println!("add exec")
                }
            };
        }
        Ok(())
    }
}

#[cfg(feature = "client")]
impl NavigatorProtocols<Protocols> {

    pub async fn resolve_protocol(&self,assets:&Arc<StaticAssetsCollection>,write:&mut WriteSender,read:&mut ReadSender)
    {
        if let Some(protocol) = self.protocols.get() {
            match protocol {
                Protocols::Read => {
                    if let Some( message) = read.next().await {
                        let query = message.unwrap_or(Message::binary(Vec::new())).into_text().unwrap_or(String::from(""));
                        assets.search(query, write).await;
                    }
                },
                Protocols::Write => {
                    assets.write_all(write).await;
                },
                Protocols::ExecJS => {
                    if let Some( message) = read.next().await {
                        let query = message.unwrap_or(Message::binary(Vec::new())).into_text().unwrap_or(String::from(""));
                        assets.exec(query, write).await;
                    }
                }
            }
        }
    }
}


fn error_handle_set_oncelock<T>(_:T)->ErrorResponse
{
    ErrorResponse::new(Some(String::from("can't reset data")))
}

#[cfg(feature = "client")] 
pub async fn handle_client(stream:TcpStream,assets:&Arc<StaticAssetsCollection>) 
{
    let stream_navigator:NavigatorProtocols<Protocols> = NavigatorProtocols::new();
    if let Ok(ws) = accept_hdr_async(stream, stream_navigator.hands_shake_callback()).await {
        let (mut write,mut read) = ws.split(); 
        stream_navigator.resolve_protocol(assets,&mut write,&mut read).await;
    } else {

        todo!()
    } 
}

#[cfg(feature = "client")]
pub fn client_resolve_directories<F>(path:&Path,handler:&mut F)->Result<(), Box<dyn Error>> 
    where 
        F: FnMut(&Path)-> Result<(), Box<dyn Error>> 
{
    for entry in get_entries(path)?.iter() {
        if entry.file_type()?.is_file() {
            handler(entry.path().leak())?;
        } else {
            if let Some(index) = get_entries(&entry.path())?.into_iter().find(|file|{OsString::from("index.qcow") == file.file_name()}) {
                handler(index.path().leak())?;
            }
        }
    }
    Ok(())
}


#[cfg(feature = "deamon")]
pub async fn handle_deamon(stream:TcpStream)->Result<(),TungError>
{
    let navigator:NavigatorProtocols<CommandProtocols> = NavigatorProtocols::new();
    let ws_stream = accept_hdr_async(stream,navigator.hands_shake_callback()).await?;
    let (_,mut read) = ws_stream.split(); 
    navigator.resolve_protocol(&mut read).await?;
    // dbg!("a");
    Ok(())
}