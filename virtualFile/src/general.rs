use std::sync::{Arc,OnceLock};
use futures::StreamExt;
use tokio::net::TcpStream;
use commun_utils_handler::{IterableStringifyEnum};
use derive_utils::IterableStringifyEnum;

use tokio_tungstenite::{accept_hdr_async, 
    tungstenite::{ 
        Message, handshake::{client::Request, server::ErrorResponse}, http::{HeaderValue, Response}
    }
};

use crate::structs::iterator::{collections::StaticAssetsCollection, utils::{ReadSender, WriteSender}};

use commun_utils_handler::errors::GlobalError;

//---------------------------------------------------------------------------
//----------------------------  utils ---------------------------------------
//---------------------------------------------------------------------------



//---------------------------------------------------------------------------
//----------------------------  general enum  -------------------------------
//---------------------------------------------------------------------------


#[derive(IterableStringifyEnum)]
pub enum Protocols {
    Read,
    Write,
    ExecJS,
}

pub enum Asset {
    Cache(usize),
    Payload(usize)
}

//---------------------------------------------------------------------------
//----------------------------  Strategies ----------------------------------
//---------------------------------------------------------------------------

pub struct NavigatorProtocols
{
    protocols:OnceLock<Protocols>
}

fn error_handle_set_oncelock<T>(_:T)->ErrorResponse
{
    ErrorResponse::new(Some(String::from("can't reset data")))
}



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
                    if let Ok(protocols) = Protocols::try_from(inner) {
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

pub async fn handle_client(stream:TcpStream,assets:&Arc<StaticAssetsCollection>)
{
    let stream_navigator = NavigatorProtocols::new();
    let ws_stream = accept_hdr_async(stream, stream_navigator.hands_shake_callback()).await;
    if let Ok(ws) = ws_stream {
        let (mut write,mut read) = ws.split(); 
        stream_navigator.resolve_protocol(assets,&mut write,&mut read).await;
        // let a = [caches,payloads];
        // stream_navigator.handle_read([payloads], write, read);
        
        // stream_navigator.resolve_protocol(caches, &mut write,&mut read).await;
        // stream_navigator.resolve_protocol(payloads, &mut write,&mut read).await;
    } else {
        todo!()
    } 
}
