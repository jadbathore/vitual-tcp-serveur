use std::error::Error;

use serde::{Serialize,Deserialize};
use commun_utils_handler::{
    errors::GlobalError
};

use crate::{runtime::FakeToSubPath, structs::payloads::payload::{ReaderStrategist, TryFromReader}};

#[derive(Deserialize,Serialize, Debug)]
pub struct JsonInfo {
    url: String,
    chunks:usize,
    type_file:String,
}

impl<R:ReaderStrategist<RefPath = FakeToSubPath>> TryFromReader<R> for JsonInfo {
    type Error = Box<dyn Error>;

    fn try_from_reader(path:&R)->Result<Self,Self::Error> 
    {
        let ext =  path.extension().and_then(|a|{
            a.to_str()
        }).unwrap_or("");
        let cow_path = path.get_inner_path().get_link().to_string_lossy();
        Ok(JsonInfo { 
            url: cow_path.to_string(),
            chunks:path.chunck_number()?,
            type_file: ext.to_string()
        })
    }
}

impl JsonInfo {
    
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

    pub fn get_url(self)->String
    {
        self.url
    }

    // pub fn type_file(self)-> Cow<'a,str>


    // pub fn predict_header_capacity(&self)->Result<usize,serde_json::Error>
    // {
    //     Ok(serde_json::to_string(&self)?.as_bytes().len())
    // }
}
