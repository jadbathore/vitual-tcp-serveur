use std::{borrow::Cow, error::Error, path::Path, sync::Arc,io };


use crate::{
    structs::{
        payloads::json_struct::JsonInfo, 
        read_strategies::ReadStrategy
    }
};

#[derive(Debug)]
pub struct DataFile
{
    inner:Arc<Path>,
    payload:Arc<JsonInfo>,
    // header:JsonInfo,
    strategy:ReadStrategy
}

impl DataFile
{
    pub fn new(path:&Path)->Result<Self,Box<dyn Error>>
    {
        Ok(DataFile { 
            payload: Arc::new(JsonInfo::new(path)?),
            inner: Arc::from(path), 
            strategy: ReadStrategy::try_from(path)?
        })
    }

    // pub fn borrow_header(&self)-> &JsonInfo
    // {
    //     &self.header
    // }

    pub fn get_payload(&self)-> Arc<JsonInfo>
    {
        self.payload.clone()
    }

    pub fn get_string_lossy_url(&self)->Cow<'_, str>
    {
        self.inner.to_string_lossy()
    }
    
    pub fn get_strategy(&self)->&ReadStrategy 
    {
        &self.strategy
    }

    // pub fn predict_capacity(&self)->Result<u64,Box<dyn Error>>
    // {
    //     // 8 octets for pointer to sub vec 
    //     // 24 octets for pointer to vec bytes container

    //     let size_original = self.inner.metadata()?.len();
    //     // let header_size:u64 = self.header.predict_header_capacity()?.try_into()?;
    //     let  chunk_number:u64 = match self.strategy {
    //         ReadStrategy::Smale|ReadStrategy::Medium => {
    //             // small and mediuim = 1 chunk  
    //             return Ok(size_original + 8 + 24);
    //         },
    //         // for the large and extra 5 of ReadStrategy cases 
    //         ReadStrategy::Large => {
    //             size_original / CHUNK_SMALL_SLICE
    //         },
    //         ReadStrategy::ExtraLarge => {
    //             // large = the original chunk so 1 vector + 1 sub_vec   
    //             size_original / CHUNK_SMALL_MEDIUM
    //         }
    //         // ReadStrategy::GigaLarge=> {
    //         //     // + 1 go file are not transfert on the ram so 0 octets needed
    //         //     return Ok(0);
    //         // }
    //     };
        
    //     let sub_octet_for_pointer_sub_vec:u64 = chunk_number * 8;
    //     let sub_octet_for_pointer_vec_container:u64 = chunk_number * 24;
    //     Ok(size_original + sub_octet_for_pointer_sub_vec + sub_octet_for_pointer_vec_container)
    // }


    pub fn flush_data(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(), io::Error>
    {
        self.strategy.excute_reader_strategy(buffers, &self.inner)
        .map_err(|_|io::Error::new(io::ErrorKind::Other, "startegy can't handle reading"))?;
        Ok(())
    }
}

