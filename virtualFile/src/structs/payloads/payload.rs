use std::{borrow::Cow, error::Error, io, ops::Deref, path::Path, sync::Arc };

use fs_handler_wasi::commun_utils::{item::FileReader,read_strategies::ReadStrategy};

use crate::{
    structs::{
        payloads::json_struct::JsonInfo, 
    }
};

impl Deref for DataFile {
    type Target = FileReader;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}


#[derive(Debug)]
pub struct DataFile
{
    parent:FileReader,
    payload:Arc<JsonInfo>,
}

impl DataFile
{
    pub fn new(path:&Path)->Result<Self,Box<dyn Error>>
    {
        Ok(DataFile { 
            parent: FileReader::new(path)?,
            payload: Arc::new(JsonInfo::new(path)?),
        })
    }

    pub fn get_payload(&self)-> Arc<JsonInfo>
    {
        self.payload.clone()
    }
}
