use std::{ffi::{OsStr, OsString}, fs, path::PathBuf, sync::{Arc,OnceLock}, vec};
use futures::StreamExt;
use regex::Regex;
use serde::de::Error;
use tokio::net::TcpStream;
use commun_utils_handler::{IterableStringifyEnum};
use derive_utils::IterableStringifyEnum;
use std::path::Path;
use tokio_tungstenite::{accept_async, accept_hdr_async, tungstenite::{ 
        Message, handshake::{client::Request, server::ErrorResponse}, http::{HeaderValue, Response}
    }
};
use tokio_tungstenite::tungstenite::Error as TungError;

#[cfg(feature = "client")]
use crate::structs::iterator::{
    collections::StaticAssetsCollection,
    utils::{ReadSender, WriteSender}
};


use std::fs::File;

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

#[cfg(feature = "client")]
pub enum Asset {
    Cache(usize),
    Payload(usize)
}

//---------------------------------------------------------------------------
//----------------------------  Strategies ----------------------------------
//---------------------------------------------------------------------------

#[cfg(feature = "client")]
pub struct NavigatorProtocols
{
    protocols:OnceLock<Protocols>
}

#[cfg(feature = "client")]
impl NavigatorProtocols {

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
                    if let Ok(protocols) = Protocols::from_str(inner) {
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
    let stream_navigator = NavigatorProtocols::new();
    if let Ok(ws) = accept_hdr_async(stream, stream_navigator.hands_shake_callback()).await {
        let (mut write,mut read) = ws.split(); 
        stream_navigator.resolve_protocol(assets,&mut write,&mut read).await;
    } else {
        todo!()
    } 
}


fn error_url<T>(_:T)->TungError
{
    TungError::Utf8
}







#[cfg(feature = "deamon")]
pub async fn handle_deamon(stream:TcpStream)->Result<(),TungError>
{
    let ws_stream = accept_async(stream).await?;
        let (mut write,mut read) = ws_stream.split(); 
    if let Some(Ok(message)) = read.next().await {
        use std::borrow::Cow;

        use regex::RegexSet;

        let mut file:String = message.into_text()?;
        let regexes = [r"\.+\/",r"\/+"," "];
        let regex_set = RegexSet::new(regexes).map_err(error_url)?;
        for index in regex_set.matches(&file).iter() {
            let replacement = match index {
                0 => "/",
                1 => "-",
                _ => ""
            };

            let regex = Regex::new(regexes[index]).unwrap();
            file = regex.replace_all(&file, replacement).to_string();
        }
        dbg!(file);
        // let file = message.into_text().unwrap_or(String::from(""));

        // if let Ok(file_path) = 
        // let file = File::create(Path::new(&file))?;
        // while let Some(Ok(message)) = read.next().await  {
        //     dbg!(message);
        // }
    }
    
    Ok(())
} 