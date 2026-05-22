use std::{
    borrow::Cow, error::Error, fs::{self, DirEntry}, io::{self, BufReader, Read}, ops::Deref, path::Path, sync::Arc
};

use crate::errors::GlobalError;

pub const MEDIUM_FILE: u64 = 64 * 1024;
pub const LARGE_FILE: u64 = 10 * 1024 * 1024;
pub const HUGE_FILE: u64 = 100 * 1024 * 1024;
pub const GIGA_FILE: u64 = 1 * 1024 * 1024 * 1024;

pub const CHUNK_SMALL_SLICE:usize = 128 * 1024;
pub const CHUNK_MEDIUM_SLICE:usize =  CHUNK_SMALL_SLICE * 2;

//-------------------------------------------------------------------------
//----------------------------read-Strategies------------------------------
//-------------------------------------------------------------------------

pub trait TcpReaderStrategy where
    Self:AsRef<Path>
{
}

pub trait ReadSyncStrategies<'callback,'buffers>
    where
        Self:AsRef<Path>,
        'buffers:'callback
{
    fn use_across_file(&self,callback:Box<dyn FnMut(Arc<[u8]>) + 'callback>)->Result<(),Box<dyn Error>>;
    fn flush(&self,buffers:&'buffers mut Vec<Arc<[u8]>>)->Result<(),Box<dyn Error>>
    {
        let callback= |chunck| {
            buffers.push(chunck);
        };
        self.use_across_file(Box::new(callback))
    }
}

struct SmaleSyncRead<'cow> {
    inner:Cow<'cow,Path>
}

impl<'cow> AsRef<Path> for SmaleSyncRead<'cow> {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl<'cow> SmaleSyncRead<'cow> {
    fn new(entry: &'cow Path) -> Self
    {
        SmaleSyncRead {inner:Cow::from(entry)}
    }
}

impl<'cow,'buffers> ReadSyncStrategies<'cow,'buffers> for SmaleSyncRead<'cow> where 'buffers:'cow {
    fn use_across_file(&self,mut callback:Box<dyn FnMut(Arc<[u8]>) + 'cow>)->Result<(),Box<dyn Error>> {
        let data = fs::read(&self.inner)?;
        callback(Arc::from(data));
        Ok(())
    }
}

struct MediumSyncRead<'cow> {
    inner:Cow<'cow,Path>,
    capacity:usize
}

impl<'cow> AsRef<Path> for MediumSyncRead<'cow> {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl<'cow> MediumSyncRead<'cow> {
    fn new(entry: &'cow Path) -> Result<Self, Box<dyn Error>>
    {
        let capacity = <u64 as TryInto<usize>>::try_into(entry.metadata()?.len()).map_err(|err|{
            GlobalError::from(err)
        })?;
        Ok( MediumSyncRead { inner: Cow::from(entry) ,capacity:capacity } )
    }
}

impl<'cow,'buffers> ReadSyncStrategies<'cow,'buffers> for MediumSyncRead<'cow> where 'buffers:'cow {
    fn use_across_file(&self,mut callback:Box<dyn FnMut(Arc<[u8]>) + 'cow>)->Result<(),Box<dyn Error>> {
        let data = fs::File::open(&self.inner)?;
        let mut sub_buf:Vec<u8> = Vec::with_capacity(self.capacity);
        let mut reader = BufReader::new(data);
        reader.read_to_end(&mut sub_buf)?;
        callback(Arc::from(sub_buf));
        Ok(())
    }
}

struct ChunckSyncRead<'cow> {
    inner:Cow<'cow,Path>,
    chunck_size:usize,
}

impl<'cow> ChunckSyncRead<'cow> {

    fn new(entry:&'cow Path,chunk_size:usize)->Self
    {
        ChunckSyncRead {inner: Cow::from(entry), chunck_size:chunk_size}
    }
}

impl<'cow> AsRef<Path> for ChunckSyncRead<'cow> {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl<'cow,'buffers> ReadSyncStrategies<'cow,'buffers> for ChunckSyncRead<'cow> where 'buffers:'cow{
    fn use_across_file(&self,mut callback:Box<dyn FnMut(Arc<[u8]>) + 'cow>)->Result<(),Box<dyn Error>> {
        let data = fs::File::open(&self.inner)?;
        let mut sub_capacity_buffer = vec![0;self.chunck_size];
        let mut reader = BufReader::new(data);
        loop {
            let byte_read = reader.read(&mut sub_capacity_buffer)?;
            if byte_read == 0 {
                break;
            }
            callback(Arc::from(&sub_capacity_buffer[..byte_read]));
        }
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub enum ReadStrategy {
    Smale,
    Medium,
    Large,
    ExtraLarge,
    // GigaLarge
}

impl<'buffer> TryFrom<&'buffer Path> for ReadStrategy {
    type Error = io::Error;
    fn try_from(entry: &'buffer Path) -> Result<Self,Self::Error> {
        match entry.metadata()?.len() {
            x if x <= MEDIUM_FILE => Ok(ReadStrategy::Smale),
            x if x <= LARGE_FILE => Ok(ReadStrategy::Medium),
            x if x <= HUGE_FILE => Ok(ReadStrategy::Large),
            x if x <= GIGA_FILE  => Ok(ReadStrategy::ExtraLarge),
            _ => Err(io::Error::new(io::ErrorKind::FileTooLarge, "can't read file to large"))
        }
    }
}


// impl ReadStrategy {

//     fn get_boxed_dyn_reader<'buffer,'callback>(&self,path:&'callback Path)->Result<Box<dyn ReadSyncStrategies<'callback,'buffer> + 'callback>,Box<dyn Error>>
//         where
//             'buffer:'callback
//     {
//         let result:Box<dyn ReadSyncStrategies> = match &self {
//             Self::Smale => Box::new(SmaleSyncRead::new( &path)),
//             Self::Medium => Box::new(MediumSyncRead::new(&path)?),
//             Self::Large => Box::new(ChunckSyncRead::new(&path, CHUNK_SMALL_SLICE)),
//             Self::ExtraLarge => Box::new(ChunckSyncRead::new(&path, CHUNK_MEDIUM_SLICE)),
//         };
//         Ok(result)
//     }
// }

pub fn get_entries(path:&Path)-> Result<Vec<DirEntry>,io::Error>
{
    fs::read_dir(path)?.collect()
}

pub fn recursive_file_read<F>(path:&Path,handler:&mut F)->Result<(), Box<dyn Error>>
    where
        F: FnMut(&Path)-> Result<(), Box<dyn Error>>
{
    for entry in get_entries(path)?.iter() {
        if entry.file_type()?.is_file() {
            handler(entry.path().leak())?;
        } else {
            recursive_file_read::<F>(entry.path().leak(), handler)?;
        }
    }
    Ok(())
}


#[derive(Debug,Clone)]
pub struct FileReader<P:AsRef<Path>>
{
    inner:Arc<P>,
    strategy:ReadStrategy
}

// impl<P:AsRef<Path>> AsRef<Path> for FileReader<P> {
//     fn as_ref(&self) -> &Path {
//         self.inner
//     }
// }

// impl<T:AsRef<Path>> TryFrom<T> for FileReader {

//     type Error = Box<dyn Error>;

//     fn try_from(path:T) -> Result<Self, Self::Error> {
//         Ok(FileReader {
//             inner: Arc::from(path.as_ref()),
//             strategy: ReadStrategy::try_from(path.as_ref())?
//         })
//     }
// }


// trait ReaderCallable {}

// type f = impl FnMut(Arc<[u8]>);

// impl ReaderCallable for f {}

// impl ReaderStrategist for FileReader {
//     fn flush_data(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(), io::Error>
//     {
//         let dyn_reader = self.get_boxed_dyn_reader(&self.inner)
//         .map_err(|_|io::Error::new(io::ErrorKind::Other, "strategy can't handle reading"))?;
//         dyn_reader.flush(buffers).map_err(|_|io::Error::new(io::ErrorKind::Other, "can't flush data"))?;
//         Ok(())
//     }

//     fn use_accross_data(&self,mut_callback:impl )->Result<(), Box<dyn Error>>
//     {
//         let dyn_reader = self.get_boxed_dyn_reader(&self.inner)?;
//         dyn_reader.use_across_file(Box::new(mut_callback))?;
//         Ok(())
//     }
// }


impl<P:AsRef<Path>> Deref for FileReader<P> {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().as_ref()
    }
}

impl<'a,P:AsRef<Path>> TryFrom<&'a Path> for FileReader<P> where Box<P>: From<&'a Path> {

    type Error = Box<dyn Error>;

    fn try_from(path:&'a Path) -> Result<Self, Self::Error> {

        let a = Box::from(path);
        Ok(FileReader {
            inner: Arc::from(a),
            strategy: ReadStrategy::try_from(path)?
        })
    }
}



impl<P:AsRef<Path>> FileReader<P>
{

    pub fn get_inner_path(&self)-> &P
    {
        &self.inner
    }

    // pub fn get_string_lossy_url(&self)->Cow<'_, str>
    // {
    //     self.inner.to_string_lossy()
    // }

    pub fn get_strategy(&self)->&ReadStrategy
    {
        &self.strategy
    }

    // pub fn size(&self)->Result<u64,io::Error>
    // {
    //     Ok(self.inner.metadata()?.len())
    // }

    fn get_boxed_dyn_reader<'buffer,'callback>(&self,path:&'callback Path)->Result<Box<dyn ReadSyncStrategies<'callback,'buffer> + 'callback>,Box<dyn Error>>
        where
            'buffer:'callback
    {
        let result:Box<dyn ReadSyncStrategies> = match self.strategy {
            ReadStrategy::Smale => Box::new(SmaleSyncRead::new( &path)),
            ReadStrategy::Medium => Box::new(MediumSyncRead::new(&path)?),
            ReadStrategy::Large => Box::new(ChunckSyncRead::new(&path, CHUNK_SMALL_SLICE)),
            ReadStrategy::ExtraLarge => Box::new(ChunckSyncRead::new(&path, CHUNK_MEDIUM_SLICE)),
        };
        Ok(result)
    }

    pub fn flush_data(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(), io::Error>
    {
        let dyn_reader = self.get_boxed_dyn_reader(self)
        .map_err(|_|io::Error::new(io::ErrorKind::Other, "strategy can't handle reading"))?;
        dyn_reader.flush(buffers).map_err(|_|io::Error::new(io::ErrorKind::Other, "can't flush data"))?;
        Ok(())
    }

    pub fn use_accross_data(&self,mut_callback:impl FnMut(Arc<[u8]>))->Result<(), Box<dyn Error>>
    {
        let dyn_reader = self.get_boxed_dyn_reader(self)?;
        dyn_reader.use_across_file(Box::new(mut_callback))?;
        Ok(())
    }

}