#[allow(warnings)]
mod bindings;
pub mod structs;
mod traits;
mod enums;
mod strategy;
mod buffers;


use std::path::{Path, PathBuf};

use crate::{
    // bindings::exports::component::fs_handler_wasi::types::{ GuestBuffercollection}, 
    // bindings::component::fs_handler_wasi::types::Buffercollection,
    structs::{collection::PayloadCollection, fs_resolve::Resolve, json_struct::{self, JsonInfo}, payload_request::{self,  DataFile}}, traits::fs_read::FileReader
};
use bindings::Guest;
struct Component;


// struct BufferCollectionImpl {
//     data:&'static[u8]
// }

// impl GuestBuffercollection for BufferCollectionImpl {

//     fn get(&self) -> Vec<u8> {
//         self.data.to_vec()
//     }
// }



impl Guest for Component {

    fn resolve_request()-> Vec<String>
    {
        let path = PathBuf::from("./fs");
        let mut directories = Vec::new();
        <Resolve as FileReader>::recursive(&path,&mut directories).expect("error reading files");
        println!("directories content resolve");
        directories
    }

    fn file_handle_payloads(buffer: Vec<String>) -> Vec<String> 
    {
        buffer.iter()
        .filter(|x|{
            *x != "./fs/fs/asset/.DS_Store"
        }).map(|a|{
            let path:&Path = Path::new(a);
            let a= JsonInfo::new(path).expect("payload failed");
            serde_json::to_string(&a).expect("serialisation failed")
        }).collect()
    }

    fn file_handle_data(buffer: Vec<String>)->Vec<Vec<u8>>
    {
        // println!("handling of the files ");
        // let test:Vec<Buffercollection> = Vec::new();
        // todo!()
        // BUFFERS.set(NEXT_ID,);
        // NEXT_ID.;


        let mut datas:Vec<DataFile<'_>> = buffer.iter()
        .filter(|x|{
            *x != "./fs/fs/asset/.DS_Store"
        }).map(|i|{
            let path = Path::new(i);
            DataFile::new(path).expect("data read failed")
        }).collect();

        let payload_mut_collection = PayloadCollection::from(&mut datas);
        let mut binder = payload_mut_collection.iter();

        loop {
            if let Some(item) = binder.next(){
                binder.add_capacity_to_allocate(item.predict_capacity().expect("predict error"));
            } else {
                break;
            }
        };

        let mut buffers:Vec<Vec<u8>> = Vec::with_capacity(binder.try_get_capacity_allocate().expect("try into failed"));
        binder.reverse();
        loop {
            if let Some(item) = binder.next(){
                item.flush_data(&mut buffers).expect("data");
                // binder.add_capacity_to_allocate(item.predict_capacity().expect("predict error"));
            } else {
                break;
            }
        }
        buffers
    }
}

bindings::export!(Component with_types_in bindings);
