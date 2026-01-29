use core::panic;
use std::{borrow::Cow, error::Error, fs, path::{self, Path}, sync::{Arc, OnceLock}};
use lazy_static::lazy_static;
use time::Time;
use tokio::{net::TcpListener,runtime::Runtime};
use fs_handler_wasi::structs::{collection, json_struct::JsonInfo, payload_request};
use tokio_tungstenite::tungstenite::http::request;

use crate::{enums::errors::GlobalError, structs::{builder::{director::Director, wasi::WasiBuild}, iterator::TcpSharedContentCollection}, traits::builder::WasiUtilsBuild};

mod general;
mod structs;
mod enums;
mod general_macros;
mod traits;

// thread_local! {
//     static BUFFERS: RefCell<Vec<Vec<u8>>> = RefCell::new(panic!("no buffer provided"));

//     // static NEXT_ID: RefCell<u32> = RefCell::new(0);
// }


lazy_static!(
    static ref VFS_DIR:OnceLock<String> = OnceLock::new();
    static ref PROTOCOLS:[&'static str;2] = ["write","read"];
    static ref PAYLOADS:OnceLock<Vec<Arc<Cow<'static,str>>>> = OnceLock::new();
    static ref JSON_PAYLOADS:OnceLock<Vec<JsonInfo<'static>>> = OnceLock::new();
    static ref BUFFERS:OnceLock<Vec<Arc<[u8]>>> = OnceLock::new(); 
    static ref DATA_COLLECTION:OnceLock<TcpSharedContentCollection> = OnceLock::new();
);



// async fn runtime_tcp(result:Vec<String>)->Result<(),Box<dyn std::error::Error>>
// {
//     let mut wasi_handle_builder:WasiBuild<(Vec<String>,), (Vec<Vec<u8>>,)> = WasiBuild::default();
//     Director::construct_wasi(&mut wasi_handle_builder)?;
//     let typed_handle = wasi_handle_builder.build( "file-handle-response")?;
//     let addr = "localhost:8080";
//     let listener = TcpListener::bind(addr).await.unwrap();
//     println!("{}", "running websocket server...connect from index.html");
//     let (buffers,) = typed_handle.call(wasi_handle_builder.get_store()?, (result,))?;
//     while let Ok((stream, _)) = listener.accept().await {
//         tokio::spawn(general::handle_client(stream, &buffers));
//     }
//     Ok(())
// }



fn construct_on_preloaded_files<P,R>(param:P,func_name:&str)->Result<R, Box<dyn Error>>
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




fn main()->Result<(),Box<dyn std::error::Error>> 
{
    match std::env::var("VFS_DIR") {
        Ok(str) => {
            VFS_DIR.set(str).map_err(error_handle_set_oncelock)?;
        },
        Err(err) => {
            return Err(Box::new(err));
        }
    }
    // -----------------------------------------------------------------------------------------------------------------
    let mut wasi_req_builder:WasiBuild<(), (Vec<String>,)> = WasiBuild::default();
    Director::construct_wasi(&mut wasi_req_builder)?;
    let typed_req = wasi_req_builder.build("resolve-request")?;
    let (requests_list,) = typed_req.call(wasi_req_builder.get_store()?,())?;
    let requests_arc = Arc::new(requests_list);
    let a = requests_arc.iter().as_ref().to_vec();
    // -----------------------------------------------------------------------------------------------------------------
    //  let mut builder:WasiBuild<(Vec<String>,), (Vec<String>,)> = WasiBuild::default();
    // Director::construct_wasi(&mut builder)?;
    // let typed = builder.build("file-handle-payloads")?;
    // let a = typed.call(builder.get_store()?,result)?;

    let (payloads,)= construct_on_preloaded_files::<(Vec<String>,), (Vec<String>,)>((requests_arc.to_vec(),),"file-handle-payloads")?;
    // let json_payload:Vec<JsonInfo> = payloads.iter();
    let mut json_payload:Vec<JsonInfo> = Vec::new();

    for payload in payloads.iter() {
        json_payload.push(serde_json::from_str(payload)?);
    }
    // .map(|i|{
    //     serde_json::from_str(i).expect("error serde")
    // }).collect();
    let a:Vec<Arc<Cow<'_,str>>> = payloads.into_iter().map(|p|Arc::from(Cow::Owned(p))).collect();

    PAYLOADS.set(a).map_err(error_handle_set_oncelock)?;
    JSON_PAYLOADS.set(json_payload).map_err(error_handle_set_oncelock)?;
    let (buffers_raw,) = construct_on_preloaded_files::<(Vec<String>,), (Vec<Vec<u8>>,)>((requests_arc.to_vec(),),"file-handle-data")?;
    let buffers:Vec<Arc<[u8]>> = buffers_raw.into_iter().map(Arc::from).collect();


    BUFFERS.set(buffers).map_err(error_handle_set_oncelock)?;
    DATA_COLLECTION.set(TcpSharedContentCollection::new()?).map_err(error_handle_set_oncelock)?;


    // dbg!(TcpSharedContentCollection::new().unwrap().search("./fs/fs/test/scene1/1.Setting/resizeSetting.cjs"));
    Runtime::new()?.block_on(async {
        let addr = "localhost:8080";     
        let listener = TcpListener::bind(addr).await.unwrap();
        println!("{}", "running websocket server...connect");

        while let Ok((stream, addr)) = listener.accept().await {
            if let Some(tcp_collection) = DATA_COLLECTION.get() {
                tokio::spawn(general::handle_client(stream, tcp_collection,addr.ip()));
            }
        }
    });

    // for payload in payloads.iter() {
    //     let json_payload:JsonInfo = serde_json::from_str(payload)?;
    //     dbg!(json_payload);
    // }
    
    // let strus= serde_json::from_str()
    // let mut wasi_handle_payload_builder:WasiBuild<(Vec<String>,), (Vec<String>,)> = WasiBuild::default();
    // Director::construct_wasi(&mut wasi_handle_payload_builder)?;
    // let typed_req = wasi_handle_payload_builder.build("file-handle-payloads")?;
    // let (stringfied_payload,) = typed_req.call(wasi_handle_payload_builder.get_store()?,(result.clo,))?;
    // PAYLOADS.set(payloads).map_err(|_|{
    //     Box::new(GlobalError::ResetOnceLock)
    // })?;
    // // -----------------------------------------------------------------------------------------------------------------
    // let (buffers,) = construct_on_preloaded_files::<(Vec<String>,), (Vec<Vec<u8>>,)>(result,"file-handle-data")?;

    // // let mut wasi_handle_data_builder:WasiBuild<(Vec<String>,), (Vec<Vec<u8>>,)> = WasiBuild::default();
    // // Director::construct_wasi(&mut wasi_handle_data_builder)?;
    // // let typed_handle = wasi_handle_data_builder.build( "file-handle-data")?;
    // // let (buffers,) = typed_handle.call(wasi_handle_data_builder.get_store()?, (result,))?;

    // BUFFERS.set(buffers).map_err(|_|{
    //     Box::new(GlobalError::ResetOnceLock)
    // })?;
    // // -----------------------------------------------------------------------------------------------------------------
    // Runtime::new()?.block_on(async {
    //     let addr = "localhost:8080";     
    //     let listener = TcpListener::bind(addr).await.unwrap();
    //     println!("{}", "running websocket server...connect from index.html");
    //     while let Ok((stream, _)) = listener.accept().await {
    //         if let (Some(data),Some(payloads)) = (BUFFERS.get(),PAYLOADS.get()) {
    //             tokio::spawn(general::handle_client(stream, data,payloads));
    //         } else {
    //             break;
    //         }
    //     }
    // });

    // general::read_fs(result,typed_handle, wasi_handle_builder.get_store()?)?;
    // -----------------------------------------------------------------------------------------------------------------

    // let a = Vec::with_capacity();
    // let inner = Vector.into_inner();

    // let (component,linker,engine,wasi) = build_utils("../test/")?;
    // // let vector = Vec::new(); 
    // let wasi_state_resolve = WasiState::new(wasi,ResourceTable::new());
    // let mut store_resolve  = Store::new(&engine, wasi_state_resolve);
    // let instance_resolve = build_instance(&mut store_resolve,&component,&linker)?;
    // let typed_resolve:TypedFunc<(),(Vec<String>,)> = call_wasi_resolve(instance_resolve,&mut store_resolve,"resolve-request")?;
    
    // let (result,) = typed_resolve.call(&mut store_resolve, ())?;

    // let wasi_state_response = WasiState::new(wasi.,ResourceTable::new());

    // let mut store_response  = Store::new(&engine, wasi_state_response);
    // let instance_response = build_instance(&mut store_response,&component,&linker)?;
    // let typed_handle:TypedFunc<(Vec<String>,),()> = call_wasi_resolve::<(Vec<String>,),()>(instance_response,&mut store_response, "file-handle-response")?;
    // typed_handle.call(&mut store_response, (result,))?;

    // for r in result{

    // }

    // let mut buffer = Vec::new();
    
    // read_fs(result, typed_handle, &mut store, &mut buffer);
    // pub 

    // let allocate_ram_usage = 

  
    
    Ok(())
}
