use std::sync::OnceLock;
use futures::StreamExt;
use tokio::net::TcpStream;

use tokio_tungstenite::{accept_hdr_async, 
    tungstenite::{ 
        handshake::{client::Request, server::ErrorResponse},
        http::{HeaderValue, Response},
    }
};

use crate::{
    PROTOCOLS,
    structs::{
        iterator::cached_data::{ReadSender,TcpItem, WriteSender}, 
        payloads::payload:: DataFile,
    }
};
use crate::structs::iterator::cached_data::PayloadSender;


enum Protocols {
    Read,
    Write
}


pub struct NavigatorProtocols
{
    protocols:OnceLock<Protocols>
}

fn error_handle_set_oncelock<T>(_:T)->ErrorResponse
{

    ErrorResponse::new(Some(String::from("can't reset data")))
}


impl NavigatorProtocols {

    pub fn new()->Self
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
                    if PROTOCOLS.contains(&inner) {
                        let header = inner.parse::<HeaderValue>().map_err(|_|{
                            ErrorResponse::new(Some(String::from("can't parse : ") + inner))
                        })?;
                        res.headers_mut().insert("Sec-WebSocket-Protocol",header);
                        match inner {
                            x if PROTOCOLS[0] == x => {
                                self.protocols.set(Protocols::Write).map_err(error_handle_set_oncelock)?;
                            },
                            x if PROTOCOLS[1] == x => {
                                self.protocols.set(Protocols::Read).map_err(error_handle_set_oncelock)?;
                            }
                            _ => ()
                        }
                    } else {
                        return Err(ErrorResponse::new(Some(String::from("protocole :") + inner + "not accept" )));
                    }
                }
            Ok(res)
        }
    }

    pub async fn resolve_protocol<PS: PayloadSender + 'static>(&self,sender:Box<PS>,write:&mut WriteSender,read:&mut ReadSender)
    {
        if let Some(protocol) = self.protocols.get() {
            match protocol {
                Protocols::Read => {
                    sender.read_splitsink(write, read).await;
                },
                Protocols::Write => {
                    sender.write_splitsink(write).await;
                }
            }
        }
    }
}

pub async fn handle_client<P,C>(stream:TcpStream,payloads:Box<P>,caches:Box<C>)
    where 
        P:PayloadSender<Item = &'static DataFile> + 'static,
        C:PayloadSender<Item = TcpItem> + 'static 
{
    let stream_navigator = NavigatorProtocols::new();
    let ws_stream = accept_hdr_async(stream, stream_navigator.hands_shake_callback()).await;
    if let Ok(ws) = ws_stream {
        let (mut write,mut read) = ws.split(); 
        stream_navigator.resolve_protocol(payloads, &mut write,&mut read).await;
        stream_navigator.resolve_protocol(caches, &mut write,&mut read).await;
    } else {
        todo!()
        
    } 
}
