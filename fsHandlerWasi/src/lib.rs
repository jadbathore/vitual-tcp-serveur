#[allow(warnings)]
mod bindings;
pub mod structs;
mod traits;
mod enums;
mod strategy;
mod buffers;

// use std::{fs::{self, DirEntry, File}, path::{Path, PathBuf}};
use lazy_static::lazy_static;
use std::sync::{Arc,OnceLock};
use virtualFile;
// use crate::{
//     // bindings::exports::component::fs_handler_wasi::types::{ GuestBuffercollection}, 
//     // bindings::component::fs_handler_wasi::types::Buffercollection,
//     structs::{collection::PayloadCollection, fs_resolve::Resolve, json_struct::{self, JsonInfo}, payload_request::{self,  DataFile}}, traits::fs_read::FileReader
// };

use bindings::Guest;
struct Component;


lazy_static! (
    static ref CACHE_PAYLOADS:Arc<CacheCollection> = OnceLock::new();
    static ref PAYLOADS:Arc<PayloadCollection> =OnceLock::new();
);



impl Guest for Component {

    // test d'observation des fichiers et comportement 
    fn ta0043()
    {
        
    }

}

bindings::export!(Component with_types_in bindings);
