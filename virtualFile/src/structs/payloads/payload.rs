#[cfg(feature = "deamon")]
use std::path::PathBuf;
use std::{error::Error, ops::Deref, path::Path, sync::Arc };
use commun_utils_handler::fs_strategies::FileReader;
// use fs_handler_wasi::commun_utils::{item::FileReader,read_strategies::ReadStrategy};

use crate::structs::{
        async_strategies::FileAsyncReader, payloads::json_struct::JsonInfo 
    };

pub trait ReaderStrategist where Self:AsRef<Path> {}

impl ReaderStrategist for FileAsyncReader {}
impl ReaderStrategist for FileReader {}


#[derive(Debug)]
pub struct DataFile<R:ReaderStrategist>
{
    parent:R,
    payload:Arc<JsonInfo>,
}

// impl<T:ReaderStrategist> Into<T> for Path {
//     fn into(&self) -> T {
//         T::from(self)
//     }
// } 

// impl<'path,R:ReaderStrategist> TryFrom<&'path Path> for DataFile<R> {
//     type Error = io::Error;

//     fn try_from(value: &'path Path)->Result<Self,Self::Error>
//     {
//         Ok(
//             DataFile { parent: value.into(), payload: Arc::new(JsonInfo::new(value)?) }
//         )
//         // parent:file
//     }
// }




impl<R:ReaderStrategist> DataFile<R>
{
    pub fn new(reader_strategy:R)->Result<Self,Box<dyn Error>>
    {
        let a = reader_strategy.as_ref().to_path_buf();
        Ok(DataFile { 
            parent: reader_strategy,
            payload: Arc::new(JsonInfo::new(a.as_path())?),
        })
    }
    pub fn get_payload(&self)-> Arc<JsonInfo>
    {
        self.payload.clone()
    }
}

impl<R:ReaderStrategist> Deref for DataFile<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}



// #[derive(Debug)]
// pub struct CloudFile
// {
//     inner:Arc<Path>,
//     payload:Arc<JsonInfo>,
// }



