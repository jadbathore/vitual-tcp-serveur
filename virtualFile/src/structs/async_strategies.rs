use std::{
    borrow::Cow, error::Error, io::{self, Read}, ops::Deref, path::{self, Path}, pin::Pin, sync::Arc
};

use commun_utils_handler::{
    errors::GlobalError,
    fs_strategies::{
        CHUNK_MEDIUM_SLICE, CHUNK_SMALL_SLICE, GIGA_FILE, HUGE_FILE, LARGE_FILE, MEDIUM_FILE, ReadStrategy
    },
};
use futures::lock::Mutex;

use crate::structs::storage::BoxFuture;
use tokio::{fs, io::AsyncReadExt};

use tokio::io::BufReader;

pub type MutableBoxedFuture<'a,T> = Pin<Box<dyn Future<Output = T>  + 'a>>;

// type MutableBoxedFuture<'callback,A,R> = Box<dyn FnMut(A)->Pin<Box<dyn Future<Output = Result<R,Box<dyn Error>>>>> + Send + 'callback>;


// pub const MEDIUM_FILE: u64 = 64 * 1024;
// pub const LARGE_FILE: u64 = 10 * 1024 * 1024;
// pub const HUGE_FILE: u64 = 100 * 1024 * 1024;
// pub const GIGA_FILE: u64 = 1 * 1024 * 1024 * 1024; 
 
// pub const CHUNK_SMALL_SLICE:usize = 128 * 1024;
// pub const CHUNK_MEDIUM_SLICE:usize =  CHUNK_SMALL_SLICE * 2;

//-------------------------------------------------------------------------
//----------------------------read-Strategies------------------------------
//-------------------------------------------------------------------------


pub trait ReadAsyncStrategies where Self:AsRef<Path> {
    fn use_across_file<'callback>(self:Arc<Self>,callback:MutableBoxedFuture<'callback,Result<(),io::Error>>)->BoxFuture<'callback,Result<(),io::Error>>;
    fn flush<'buffers>(self:Arc<Self>,mut mutex_buffers:Mutex<Vec<Arc<[u8]>>>)->BoxFuture<'buffers,Result<(),io::Error>> {
        tokio::spawn(async move {
            
        });
        self.use_across_file(Box::pin(async move {
            let buffers = mutex_buffers.get_mut();
            buffers.push(Arc::from(Vec::new()));
            Ok(())
        }))
    }

}

// pub trait ReadAsyncStrategies
//     where 
//         Self:AsRef<Path>,
// {
//     fn use_across_file<'callback>(self,callback:MutableBoxedFuture<'callback,Arc<[u8]>,()>)->BoxFuture<'callback,Result<(),io::Error>>;
//     fn flush<'buffers>(self:Arc<Self>,buffers:Mutex<Vec<Arc<[u8]>>>)->BoxFuture<'buffers,Result<(),io::Error>>
//     {

//         // let callback:MutableBoxedFuture<'_,Arc<[u8]>,()> = Box::new(&mut |chunck| {
//         //     Box::pin(async move { 
//         //         let mut mutable = buffers.get_mut();
//         //         mutable.push(chunck);
//         //         Ok(())
//         //     })
//         // });
//         let call = Box::new(&mut |chunck| {
//             let buffer = buffers;
                
//             async fn call(mut buf:Mutex<Vec<Arc<[u8]>>>,chunck:Arc<[u8]> ) -> Result<(),io::Error>{
//                 let mutable = buf.get_mut();
//                 mutable.push(chunck);
//                 Ok(())
//             }
//             let a 
//             Box::pin(call(buffers,chunck))
//         });
//         self.use_across_file(call)
//     }
// }

struct SmaleAsyncRead {
    inner:Arc<Path>
}


impl AsRef<Path> for SmaleAsyncRead {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl SmaleAsyncRead {
    fn new(entry: &Path) -> Self
    {
        SmaleAsyncRead { inner:Arc::from(entry) } 
    }
}

impl ReadAsyncStrategies for SmaleAsyncRead  {
    fn use_across_file<'callback>(self:Arc<Self>,mut callback:MutableBoxedFuture<'callback,()>)->BoxFuture<'callback,Result<(),io::Error>>
    {
        Box::pin(async move {
            let data = fs::read(self.as_ref()).await?;
            Box::pin(callback(Arc::from(data)));
            Ok(())
        })
    }
}

struct MediumAsyncRead {
    inner:Arc<Path>,
    capacity:usize
}

impl AsRef<Path> for MediumAsyncRead {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl MediumAsyncRead {
    fn new(entry: &Path) -> Result<Self, Box<dyn Error>> 
    {
        let capacity = <u64 as TryInto<usize>>::try_into(entry.metadata()?.len()).map_err(|err|{
            GlobalError::from(err)
        })?;
        Ok( MediumAsyncRead { inner: Arc::from(entry) ,capacity:capacity } )
    }
}

impl ReadAsyncStrategies for MediumAsyncRead {
    fn use_across_file<'callback>(self:Arc<Self>,mut callback:MutableBoxedFuture<'callback,()>)->BoxFuture<'callback,Result<(),io::Error>>
    {
        Box::pin(async move {
            let data = fs::File::open(&self.inner).await?;
            let mut sub_buf:Vec<u8> = Vec::with_capacity(self.capacity); 
            let mut reader = BufReader::new(data);
            reader.read_to_end(&mut sub_buf).await?;
            // reader.read_to_end(&mut sub_buf)?;
            callback(Arc::from(sub_buf));
            Ok(())
        })
        
        // Ok(())
    }
}

struct ChunckAsyncRead {
    inner:Arc<Path>,
    chunck_size:usize,
}

impl ChunckAsyncRead{
    
    fn new(entry:&Path,chunk_size:usize)->Self
    { 
        ChunckAsyncRead {inner: Arc::from(entry), chunck_size:chunk_size}
    }
}

impl<'cow> AsRef<Path> for ChunckAsyncRead {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}


impl ReadAsyncStrategies for ChunckAsyncRead {
    fn use_across_file<'callback>(self:Arc<Self>,mut callback:MutableBoxedFuture<'callback,Arc<[u8]>,()>)->BoxFuture<'callback,Result<(),io::Error>>
    {
        Box::pin(async move {
            let data = fs::File::open(&self.inner).await?; 
            let mut sub_capacity_buffer:Vec<u8> = vec![0;self.chunck_size];
            let mut reader = BufReader::new(data);
            // let byte_read = reader.read(&mut sub_capacity_buffer).await?;
            loop {
                let byte_read = reader.read(&mut sub_capacity_buffer).await?;
                if byte_read == 0 {
                    break;
                }
                callback(Arc::from(&sub_capacity_buffer[..byte_read]));
                }
            Ok(())
        })
        // let data = fs::File::open(&self.inner); 
        // let mut sub_capacity_buffer = vec![0;self.chunck_size];
        // let mut reader = BufReader::new(data);
        // loop {
        //     let byte_read = reader.read(&mut sub_capacity_buffer)?;
        //     if byte_read == 0 {
        //         break;
        //     }
        //     callback(Arc::from(&sub_capacity_buffer[..byte_read]));
        // }
        // Ok(())
    }
}





// impl ReadStrategy {

//     fn get_dyn_reader<'buffer,'callback>(&self,path:&'callback Path)->Result<Box<dyn ReadSyncStrategies<'callback,'buffer> + 'callback>,Box<dyn Error>> 
//         where 
//             'buffer:'callback
//     {
//         let result:Box<dyn ReadSyncStrategies> = match &self {
//             Self::Smale => Box::new(SmaleSyncRead::new( &path)),
//             Self::Medium => Box::new(MediumAsyncRead::new(&path)?),
//             Self::Large => Box::new(ChunckAsyncRead::new(&path, CHUNK_SMALL_SLICE)),
//             Self::ExtraLarge => Box::new(ChunckAsyncRead::new(&path, CHUNK_MEDIUM_SLICE)),
//         };
//         Ok(result)
//     }
// }


#[derive(Debug,Clone)]
pub struct FileAsyncReader
{
    inner:Arc<Path>,
    strategy:ReadStrategy
}

impl Deref for FileAsyncReader {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}


impl<'a> TryFrom<&'a Path> for FileAsyncReader {

    type Error = Box<dyn Error>;

    fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
        Ok(FileAsyncReader { 
            inner: Arc::from(path), 
            strategy: ReadStrategy::try_from(path)?
        })
    }
}

impl FileAsyncReader
{
    pub fn get_string_lossy_url(&self)->Cow<'_, str>
    {
        self.inner.to_string_lossy()
    }
    
    pub fn get_strategy(&self)->&ReadStrategy 
    {
        &self.strategy
    }

    pub fn size(&self)->Result<u64,io::Error>
    {
        Ok(self.inner.metadata()?.len())
    }

    fn get_dyn_arc_reader<'callback>(&self,path:&'callback Path)->Result<Arc<dyn ReadAsyncStrategies + 'callback>,Box<dyn Error>> {
        let result:Arc<dyn ReadAsyncStrategies> = match &self.strategy {
            ReadStrategy::Smale => Arc::from(SmaleAsyncRead::new( &path)),
            ReadStrategy::Medium => Arc::from(MediumAsyncRead::new(&path)?),
            ReadStrategy::Large => Arc::from(ChunckAsyncRead::new(&path, CHUNK_SMALL_SLICE)),
            ReadStrategy::ExtraLarge => Arc::from(ChunckAsyncRead::new(&path, CHUNK_MEDIUM_SLICE)),
        };
        Ok(result)
    }

    pub async fn flush_data(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(), io::Error>
    {
        let dyn_reader = self.get_dyn_arc_reader(&self.inner)
        .map_err(|_|io::Error::new(io::ErrorKind::Other, "strategy can't handle reading"))?;
        dyn_reader.flush(Mutex::new(buffers.to_vec())).await.map_err(|_|io::Error::new(io::ErrorKind::Other, "can't flush data"))?;
        Ok(())
    }

    pub async fn use_accross_data<'callback>(&self,mut_callback:MutableBoxedFuture<'callback,Arc<[u8]>,()>)->Result<(), Box<dyn Error>>
    {
        let dyn_reader = self.get_dyn_arc_reader(&self.inner)?;
        dyn_reader.use_across_file(mut_callback).await?;
        Ok(())
    }

}