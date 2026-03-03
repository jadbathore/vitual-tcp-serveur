use std::{borrow::Cow, error::Error, io, ops::Deref, path::Path, sync::Arc };

use crate::commun_utils::read_strategies::ReadStrategy;


#[derive(Debug)]
pub struct FileReader
{
    inner:Arc<Path>,
    strategy:ReadStrategy
}


impl<'a> TryFrom<&'a Path> for FileReader {

    type Error = Box<dyn Error>;

    fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
        Ok(FileReader { 
            inner: Arc::from(path), 
            strategy: ReadStrategy::try_from(path)?
        })
    }
}

impl FileReader
{
    pub fn new(path:&Path)->Result<Self,Box<dyn Error>>
    {
        Ok(FileReader { 
            inner: Arc::from(path), 
            strategy: ReadStrategy::try_from(path)?
        })
    }

    pub fn get_string_lossy_url(&self)->Cow<'_, str>
    {
        self.inner.to_string_lossy()
    }
    
    pub fn get_strategy(&self)->&ReadStrategy 
    {
        &self.strategy
    }

    pub fn flush_data(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(), io::Error>
    {
        self.strategy.excute_reader_strategy(buffers, &self.inner)
        .map_err(|_|io::Error::new(io::ErrorKind::Other, "strategy can't handle reading"))?;
        Ok(())
    }
}
