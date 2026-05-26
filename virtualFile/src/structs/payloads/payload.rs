#[cfg(feature = "deamon")]
use std::path::PathBuf;
use std::{error::Error, fmt::Debug, ops::Deref, path::Path, sync::Arc };
use commun_utils_handler::{errors::GlobalError, fs_strategies::{CHUNK_MEDIUM_SLICE, CHUNK_SMALL_SLICE, FileReader, ReadStrategy}};

// use fs_handler_wasi::commun_utils::{item::FileReader,read_strategies::ReadStrategy};

use crate::{
    runtime::FakeToSubPath, 
    structs::payloads::json_struct::{JsonInfo}
};

pub trait ReaderStrategist where Self: Deref<Target = Path> + AsRef<ReadStrategy> {

    type RefPath:AsRef<Path>;
    
    fn get_inner_path(&self)-> &Self::RefPath;

    fn chunck_number(&self)->Result<usize,Box<GlobalError>> {
        match self.as_ref() {
            ReadStrategy::Smale|ReadStrategy::Medium => Ok(1),
            ReadStrategy::Large => Ok(self.metadata()?.len() as usize /CHUNK_SMALL_SLICE),
            ReadStrategy::ExtraLarge => Ok(self.metadata()?.len() as usize /CHUNK_MEDIUM_SLICE)
        }
    }
}

pub trait TryFromReader<R:ReaderStrategist> where Self: Sized {
    type Error;
    fn try_from_reader(value:&R)->Result<Self,Self::Error>;
}




impl<'path,P:AsRef<Path>> ReaderStrategist for FileReader<P> {
    type RefPath = P;

    fn get_inner_path(&self)-> &Self::RefPath {
        self.get_inner_path()
    }
}

#[derive(Debug)]
pub struct DataFile<R:ReaderStrategist>
{
    parent:R,
    payload:Arc<JsonInfo>,
}

impl<R:ReaderStrategist<RefPath = FakeToSubPath>> DataFile<R>
{
    pub fn new(reader_strategy:R)->Result<Self,Box<dyn Error>>
    {
        let json = JsonInfo::try_from_reader(&reader_strategy)?;
        Ok(DataFile { 
            parent: reader_strategy,
            payload: Arc::new(json),
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
