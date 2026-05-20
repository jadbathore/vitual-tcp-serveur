use std::{
    error::Error, io ,path::Path, pin::Pin, sync::Arc
};

use commun_utils_handler::{
    errors::GlobalError,
    fs_strategies::{
        CHUNK_MEDIUM_SLICE, CHUNK_SMALL_SLICE,ReadStrategy
    },
};
use futures::lock::Mutex;
// use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

// use crate::{general::ReadSender};


use tokio::{sync::mpsc,fs, io::AsyncReadExt};

use tokio::io::BufReader;


pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;


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


pub trait ReadAsyncStrategies where Self:AsRef<Path> + Send + Sync {
    fn use_across_file<'path>(&'path self)->BoxFuture<'path,Result<mpsc::Receiver<Arc<[u8]>>,io::Error>>;
    // fn get_capacity(self:Arc<Self>)->usize;
    fn flush<'path>(&'path self,mut mutex_buffers:Mutex<Vec<Arc<[u8]>>>)->BoxFuture<'path,Result<(),io::Error>> 
        where 
            Self: 'path
    {
        Box::pin(async move {
            let mut buffer = mutex_buffers.get_mut();
            let mut rx = self.use_across_file().await?;
            rx.recv_many(&mut buffer, 1).await;
            Ok(())
        })
    }

}

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
     fn use_across_file<'callback>(&'callback self)->BoxFuture<'callback,Result<mpsc::Receiver<Arc<[u8]>>,io::Error>>
    {
        Box::pin(async move {
            let (tx,rx) = mpsc::channel(10);
            let data = fs::read(self.as_ref()).await?;
            let _ = tx.send(Arc::from(data)).await;
            // callback.await;
            // Box::pin(callback(Arc::from(data)));
            Ok(rx)
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
     fn use_across_file<'callback>(&'callback self)->BoxFuture<'callback,Result<mpsc::Receiver<Arc<[u8]>>,io::Error>>
    {
        Box::pin(async move {
            let (tx,rx) = mpsc::channel(10);
            let data = fs::File::open(&self.inner).await?;
            let mut sub_buf:Vec<u8> = Vec::with_capacity(self.capacity); 
            let mut reader = BufReader::new(data);
            reader.read_to_end(&mut sub_buf).await?;
            // reader.read_to_end(&mut sub_buf)?;
            let _ = tx.send(Arc::from(sub_buf)).await;
            // callback(Arc::from(sub_buf));
            Ok(rx)
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
     fn use_across_file<'callback>(&'callback self)->BoxFuture<'callback,Result<mpsc::Receiver<Arc<[u8]>>,io::Error>>
    {
        Box::pin(async move {
            let (tx,rx) = mpsc::channel(10);
            let data = fs::File::open(&self.inner).await?; 
            let mut sub_capacity_buffer:Vec<u8> = vec![0;self.chunck_size];
            let mut reader = BufReader::new(data);
            // let byte_read = reader.read(&mut sub_capacity_buffer).await?;
            loop {
                let byte_read = reader.read(&mut sub_capacity_buffer).await?;
                if byte_read == 0 {
                    break;
                }
                tx.send(Arc::from(&sub_capacity_buffer[..byte_read])).await
                .map_err(|err|{io::Error::new(io::ErrorKind::Interrupted,err.to_string())})?;
                }
            Ok(rx)
        })
    }
}


#[derive(Debug,Clone)]
pub struct FileAsyncReader
{
    inner:Arc<Path>,
    strategy:ReadStrategy
}

impl AsRef<Path> for FileAsyncReader {
    fn as_ref(&self) -> &Path {
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
    // pub fn get_string_lossy_url(&self)->Cow<'_, str>
    // {
    //     self.inner.to_string_lossy()
    // }
    
    // pub fn get_strategy(&self)->&ReadStrategy 
    // {
    //     &self.strategy
    // }

    // pub fn size(&self)->Result<u64,io::Error>
    // {
    //     Ok(self.inner.metadata()?.len())
    // }

    fn get_dyn_arc_reader<'callback>(&self,path:&'callback Path)->Result<Arc<dyn ReadAsyncStrategies + 'callback>,Box<dyn Error>> {
        let result:Arc<dyn ReadAsyncStrategies + 'callback> = match &self.strategy {
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

    pub async fn use_accross_data<'callback>(&self,mut callback:impl AsyncFnMut(Arc<[u8]>))->Result<(), Box<dyn Error>>
    {
        let dyn_reader = self.get_dyn_arc_reader(&self.inner)?;
        let mut rx = dyn_reader.use_across_file().await?;

        let mut buffer = Vec::with_capacity(100);
        rx.recv_many(&mut buffer, 100).await;
        for chunck in buffer {
            callback(chunck).await
        }
        Ok(())
    }

}


// pub type WriteSender = SplitSink<WebSocketStream<TcpStream>,Message>;
// async fn a<'a>(path: &'a Path,read:&mut ReadSender){
//     let file_async = FileAsyncReader::try_from(path).unwrap();
//     file_async.use_accross_data(async |value |{
//         read.next().await;

//     }).await.unwrap();
// }