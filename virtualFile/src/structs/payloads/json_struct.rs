use std::{error::Error, path::Path};

use serde::Serialize;
use fs_handler_wasi::commun_utils::{
    error::GlobalError, 
    read_strategies::{CHUNK_SMALL_MEDIUM, CHUNK_SMALL_SLICE, GIGA_FILE, HUGE_FILE, LARGE_FILE}
};

#[derive(Serialize, Debug)]
pub struct JsonInfo {
    url: String,
    size:u64,
    chunks:usize,
    type_file:String,
}

impl JsonInfo {
    pub fn new(path:&Path)->Result<Self,Box<dyn Error>>
    {
        let ext =  path.extension().and_then(|a|{
            a.to_str()
        }).unwrap_or("");
        let size = path.metadata()?.len();
        let chunck= <u64 as TryInto<usize>>::try_into(
            match size {
                // x if x >= MEDIUM_FILE => ,
                x if x <= LARGE_FILE =>  1 ,
                x if x <= HUGE_FILE => x / CHUNK_SMALL_SLICE,
                x if x <= GIGA_FILE  => x / CHUNK_SMALL_MEDIUM,
                _ => 0
            }
        ).map_err(|err|{
            GlobalError::from(err)
        })?;
        if let Some(a) = path.file_name() {
            Ok(JsonInfo { 
                url: a.to_string_lossy().to_string(),
                chunks:chunck,
                size: size,
                type_file: ext.to_string()
            })
            // let a= a.to_os_string();
        } else {
            Err(Box::new(GlobalError::UninitializedVariable))
        }
        // // let path = ;
        // Ok(JsonInfo
        // { 
        //     url:path.file_name(),
        //     chunks:size,
        //     size:path.metadata()?.len(),
        //     type_file: Cow::from(ext)
        // })
    }
    
    pub fn stringify_to_json(&self)->Result<String,Box<GlobalError>>
    {
        let json_string = serde_json::to_string(&self).map_err(|_|{
            Box::new(GlobalError::JsonSerialize)
        })?;
        Ok(json_string)
    }

    pub fn get_chunks(&self)-> usize
    {
        self.chunks
    }

    // pub fn type_file(self)-> Cow<'a,str>


    // pub fn predict_header_capacity(&self)->Result<usize,serde_json::Error>
    // {
    //     Ok(serde_json::to_string(&self)?.as_bytes().len())
    // }
}
