use std::{borrow::Cow, error::Error, path::Path };

use crate::{strategy::{CHUNK_SMALL_MEDIUM, CHUNK_SMALL_SLICE, ReadStrategy}, structs::json_struct::JsonInfo};


#[derive(Debug)]
pub struct DataFile<'a>
{
    inner:Cow<'a,Path>,
    // header:JsonInfo<'a>,
    strategy:ReadStrategy
}

impl<'a> DataFile<'a> 
{
    pub fn new(path:&'a Path)->Result<Self,Box<dyn Error>>
    {
        let read_strategy = ReadStrategy::try_from(path)?;
        let cow_path:Cow<'a, Path> = Cow::from(path);
        Ok(DataFile { 
            inner: cow_path, 
            // header: JsonInfo::new(path)?,
            strategy: read_strategy
        })
    }

    // pub fn get_header(&self)->Result<Vec<u8>, serde_json::Error>
    // {
    //     serde_json::to_vec_pretty(&self.header)
    // }

    pub fn predict_capacity(&'a self)->Result<u64,Box<dyn Error>>
    {
        // 8 octets for pointer to sub vec 
        // 24 octets for pointer to vec bytes container

        let size_original = self.inner.metadata()?.len();
        // let header_size:u64 = self.header.predict_header_capacity()?.try_into()?;
        let  chunk_number:u64 = match self.strategy {
            ReadStrategy::Smale|ReadStrategy::Medium => {
                // small and mediuim = 1 chunk  
                return Ok(size_original + 8 + 24);
            },
            // for the large and extra 5 of ReadStrategy cases 
            ReadStrategy::Large => {
                size_original / CHUNK_SMALL_SLICE
            },
            ReadStrategy::ExtraLarge => {
                // large = the original chunk so 1 vector + 1 sub_vec   
                size_original / CHUNK_SMALL_MEDIUM
            }
            ReadStrategy::GigaLarge=> {
                // + 1 go file are not transfert on the ram so 0 octets needed
                return Ok(0);
            }
        };
        
        let sub_octet_for_pointer_sub_vec:u64 = chunk_number * 8;
        let sub_octet_for_pointer_vec_container:u64 = chunk_number * 24;
        Ok(size_original + sub_octet_for_pointer_sub_vec + sub_octet_for_pointer_vec_container)
    }


    pub fn flush_data(&self,buffers:&mut Vec<Vec<u8>>)->Result<(), Box<dyn Error>>
    {
        self.strategy.excute_reader_strategy(buffers, &self.inner)?;
        Ok(())
    }
}

