use std::{borrow::Cow, error::Error, ffi::{OsStr, OsString}, io, path::Path, process::exit};

use serde::{Deserialize, Serialize};

use crate::{enums::errors::GlobalHandlerError, strategy::{CHUNK_SMALL_MEDIUM, CHUNK_SMALL_SLICE, GIGA_FILE, HUGE_FILE, LARGE_FILE, MEDIUM_FILE}};



#[derive(Serialize, Deserialize, Debug)]
pub struct JsonInfo<'a> {
    url: Cow<'a,str>,
    size:u64,
    chunks:usize,
    type_file:Cow<'a,str>,
}






impl<'a> JsonInfo<'a> {
    pub fn new<'b>(path:&'b Path)->Result<Self,Box<dyn Error>>
    where 
        'b : 'a
    {
        let ext =  path.extension().and_then(|a|{
            a.to_str()
        }).unwrap_or("");
        let size= <u64 as TryInto<usize>>::try_into(
            match path.metadata()?.len() {
                // x if x >= MEDIUM_FILE => ,
                x if x <= LARGE_FILE =>  1 ,
                x if x <= HUGE_FILE => x / CHUNK_SMALL_SLICE,
                x if x <= GIGA_FILE  => x / CHUNK_SMALL_MEDIUM,
                _ => 0
            }
        ).map_err(|err|{
            GlobalHandlerError::from(err)
        })?;
        // let path = ;
        Ok(JsonInfo
        { 
            url:path.as_os_str().to_string_lossy(),
            chunks:size,
            size:path.metadata()?.len(),
            type_file: Cow::from(ext)
        })
    }

    pub fn get_url(&'a self)-> &'a str
    {
        &self.url
        // self.url
    }

    pub fn get_chunks(&self)-> usize
    {
        self.chunks
    }

    // pub fn type_file(self)-> Cow<'a,str>


    pub fn predict_header_capacity(&self)->Result<usize,serde_json::Error>
    {
        Ok(serde_json::to_string(&self)?.as_bytes().len())
    }
}
