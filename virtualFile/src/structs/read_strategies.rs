use std::{
    borrow::Cow, 
    error::Error, 
    fs,io::{BufReader,Read}, path::Path, sync::Arc
};

use crate::{
    enums::errors::GlobalError
};

pub const MEDIUM_FILE: u64 = 64 * 1024;
pub const LARGE_FILE: u64 = 10 * 1024 * 1024;
pub const HUGE_FILE: u64 = 100 * 1024 * 1024;
pub const GIGA_FILE: u64 = 1 * 1024 * 1024 * 1024; 
pub const CHUNK_SMALL_SLICE:u64 = 128 * 1024;
pub const CHUNK_SMALL_MEDIUM:u64 =  CHUNK_SMALL_SLICE * 2;


pub trait ReadStrategies {
    // type Buffer;
    fn flush(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(),Box<dyn Error>>;
}

pub struct SmaleRead<'cow> {
    inner:Cow<'cow,Path>
}

impl<'cow> SmaleRead<'cow> {
    fn new(entry: &'cow Path) -> Self
    {
        SmaleRead {inner:Cow::from(entry)} 
    }
}

impl<'cow> ReadStrategies for SmaleRead<'cow> {
    // type Data = Vec<u8>;
    fn flush(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(),Box<dyn Error>>
    {
        let data = fs::read(&self.inner)?;
        buffers.push(Arc::from(data));
        Ok(())
    }
}

struct MediumRead<'cow> {
    inner:Cow<'cow,Path>,
    capacity:usize
}

impl<'cow> MediumRead<'cow> {
    fn new(entry: &'cow Path) -> Result<Self, Box<dyn Error>> 
    {
        let capacity = <u64 as TryInto<usize>>::try_into(entry.metadata()?.len()).map_err(|err|{
                    GlobalError::from(err)
        })?;
        Ok( MediumRead { inner: Cow::from(entry) ,capacity:capacity } )
    }
}

impl<'cow> ReadStrategies for MediumRead<'cow> {
    fn flush(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(),Box< dyn Error>> {
        let data = fs::File::open(&self.inner)?;
        let mut sub_buf:Vec<u8> = Vec::with_capacity(self.capacity); 
        let mut reader = BufReader::new(data);
        reader.read_to_end(&mut sub_buf)?;
        buffers.push(Arc::from(sub_buf));
        Ok(())
    }
}

struct ChunckRead<'cow> {
    inner:Cow<'cow,Path>,
    chunck_size:usize,
}

impl<'cow> ChunckRead<'cow> {
    fn new(entry:&'cow Path,chunk_size:usize)->Self
    { 
        ChunckRead {inner: Cow::from(entry), chunck_size:chunk_size}
    }
}

impl<'cow> ReadStrategies for ChunckRead<'cow> {
    // type Data = Vec<u8>;
    fn flush(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(),Box< dyn Error>> {
        let data = fs::File::open(&self.inner)?;
        let mut sub_capacity_buffer = Vec::with_capacity(self.chunck_size);
        let mut reader = BufReader::new(data);
        loop {
            let byte_read = reader.read(&mut sub_capacity_buffer)?;
            if byte_read == 0 {
                break;
            }
            sub_capacity_buffer.truncate(byte_read);
        }
        buffers.push(Arc::from(sub_capacity_buffer));
        Ok(())
    }
}


// struct StreamRead {
//     stream:Stdout
// }


// struct ReadNavigator<T:ReadStrategies> where  
//     T:
// {
//     strategies:T
// }

// impl<T:ReadStrategies> ReadNavigator<T> 
// {
//     pub fn new(strategy:T)->Self {
//         ReadNavigator { strategies: strategy }
//     }

//     fn read(self,buffer:&mut Vec<Arc<[u8]>>){
//         self.strategies.flush(buffer);
//     }

    
// }

// struct ReadEntry<'path> {
//     path:Cow<'path,Path>,
//     strategy:ReadStrategy
// }

// impl<'path> ReadEntry<'path>  {
//     fn new(path:Cow<'path,Path>)-> Result<Self,Box<dyn Error>>
//     {
//         let read = ReadStrategy::try_from(path.as_ref())?;
//         Ok(
//             ReadEntry { 
//                 path:path, 
//                 strategy: read
//             }
//         )
//     }
// }

#[derive(Debug)]
pub enum ReadStrategy {
    Smale,
    Medium,
    Large,
    ExtraLarge,
    // GigaLarge
}

impl<'buffer> TryFrom<&'buffer Path> for ReadStrategy {
    type Error = Box<dyn Error>;
    fn try_from(entry: &'buffer Path) -> Result<Self,Self::Error> {
        match entry.metadata()?.len() { 
            x if x <= MEDIUM_FILE => Ok(ReadStrategy::Smale),
            x if x <= LARGE_FILE => Ok(ReadStrategy::Medium),
            x if x <= HUGE_FILE => Ok(ReadStrategy::Large),
            x if x <= GIGA_FILE  => Ok(ReadStrategy::ExtraLarge),
            _ => Err(Box::new(GlobalError::FileToBig))
        }
    }
}

impl ReadStrategy {
    pub fn excute_reader_strategy<'a>(&'a self,buffer:&mut Vec<Arc<[u8]>>,path:&'a Path)-> Result<(),Box<dyn Error>>
    {
        let result:Box<dyn ReadStrategies> = match &self {
            Self::Smale => Box::new(SmaleRead::new( &path)),
            Self::Medium => Box::new(MediumRead::new(&path)?),
            Self::Large => Box::new(ChunckRead::new(&path, 128 * 1024)),
            Self::ExtraLarge => Box::new(ChunckRead::new(&path, 256 * 1024)),
            // Self::GigaLarge => todo!()
        };
        result.flush(buffer)?;
        
        // BoxedStrategy::as_ref(&result).flush(buffer);
        //  = result.as_ref();
        // let nav = ReadNavigator::new(result);
        Ok(())
    }
}

