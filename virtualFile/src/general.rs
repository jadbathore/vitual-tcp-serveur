use std::{borrow::Cow, error::Error, net::IpAddr, sync::{Arc, OnceLock}};
use futures::{SinkExt, StreamExt, stream::SplitSink};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    WebSocketStream, accept_hdr_async, 
    tungstenite::{Message, handshake::{client::Request, server::ErrorResponse},
    http::{HeaderValue, Response}, protocol::{CloseFrame, frame::coding::CloseCode},
}};
use wasmtime::{
    self, Store, 
    component::{Instance, TypedFunc}
};
use crate::{PROTOCOLS, enums::errors::GlobalError, structs::{iterator::{TcpSharedContentCollection}, states::WasiState}};



enum Protocols {
    Read,
    Write
}

impl Protocols {
    async fn protocol_handle(&self,collection:&TcpSharedContentCollection,ws_stream:WebSocketStream<TcpStream>){

        let (mut write,mut read ) = ws_stream.split();
        match self {
            Protocols::Read => {
                
                if let Some( message) = read.next().await {
                    let data = message.unwrap_or(Message::binary(Vec::new())).into_data();
                    if let Some((data,payload,_)) = collection.search(str::from_utf8(&data).unwrap_or("")) 
                    {
                        send_payload(&mut write, data, payload).await;
                    }
                }
            },
            Protocols::Write => {
                for (datas,text_payload,_) in collection.iter() {
                    send_payload(&mut write, datas, text_payload).await;
                }
            }
            
        };
        let _ = write.send(Message::Text(String::from("end"))).await;
        let _ = write.send(Message::Close(None)).await;
    } 
}

struct NavigatorProtocols
{
    protocols:OnceLock<Protocols>
}

fn error_handle_set_oncelock<T>(_:T)->ErrorResponse
{
    ErrorResponse::new(Some(String::from("can't reset data")))
}


impl NavigatorProtocols {

    fn new()->Self
    {
        NavigatorProtocols { protocols: OnceLock::new() }
    }

    fn hands_shake_callback(&self)-> impl FnOnce(&Request,Response<()>)->Result<Response<()>,ErrorResponse>
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
                            x if PROTOCOLS[1] == x =>{
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

    async fn resolve_protocol(self,collection:&TcpSharedContentCollection,ws_stream:WebSocketStream<TcpStream>)
    {
        if let Some(protocol) = self.protocols.get() {
            protocol.protocol_handle(collection, ws_stream).await;
        }
    }
}

async fn send_payload<'a>(write:&mut SplitSink<WebSocketStream<TcpStream>,Message>,datas:&[Arc<[u8]>],payload:Arc<Cow<'a,str>>)
{   
    let a = match Arc::try_unwrap(payload) {
            Ok(cow) => cow.into_owned(),
            Err(arc) => arc.to_string()
    };
    let _ = write.send(Message::Text(a)).await;

    for data in datas {
        let _ = write.send(Message::Binary(data.to_vec())).await;
    }
}

pub async fn handle_client<'a>(stream:tokio::net::TcpStream,tcp_collection:&TcpSharedContentCollection,addr:IpAddr)
{
    let stream_navigator = NavigatorProtocols::new();
    let ws_stream = accept_hdr_async(stream, stream_navigator.hands_shake_callback()).await;
    if let Ok(ws) = ws_stream {
        stream_navigator.resolve_protocol(tcp_collection, ws).await;
    } 
    let time = time::OffsetDateTime::now_utc();
    println!("data sended at {time} to {addr}");
}

pub fn call_wasi_resolve<P,R>(instance:Instance,store:&mut Store<WasiState>,func_name:&str)->Result<TypedFunc<P,R>,Box<dyn Error>>
    where 
        P: wasmtime::component::Lower + wasmtime::component::ComponentNamedList,
        R: wasmtime::component::ComponentNamedList + wasmtime::component::Lift
{
    if let Some(func) = instance.get_func(&mut *store, func_name){
        let typed:TypedFunc<P,R> = func.typed(&store)?;
        Ok(typed)
    } else {
        let msg = "unknown function ".to_string() +  func_name;
        Err(Box::new(GlobalError::ParseError(msg)))
    }
}