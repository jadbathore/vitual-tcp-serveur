use std::{
    borrow::Cow, error::Error, fs::{self, DirEntry, File, ReadDir}, io::{self, BufReader, Read, Write}, ops::Deref, path::{Path, PathBuf}, sync::Arc
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

pub trait ReadStrategies
{
    fn flush(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(),Box<dyn Error>>;
}

struct SmaleRead<'cow> {
    inner:Cow<'cow,Path>
}

impl<'cow> SmaleRead<'cow> {
    fn new(entry: &'cow Path) -> Self
    {
        SmaleRead {inner:Cow::from(entry)} 
    }
}


impl<'cow> ReadStrategies for SmaleRead<'cow> {

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
        let mut sub_capacity_buffer = vec![0;self.chunck_size];
        let mut reader = BufReader::new(data);
        loop {
            let byte_read = reader.read(&mut sub_capacity_buffer)?;
            if byte_read == 0 {
                break;
            }
            buffers.push(Arc::from(&sub_capacity_buffer[..byte_read]));
        }
        Ok(())
    }
}

struct HashDirectoryRead<'cow> 
{
    inner:&'cow Path,
}

impl<'dir> TryFrom<&'dir Path> for HashDirectoryRead<'dir> {
    type Error = Box<GlobalError>;

    fn try_from(value: &'dir Path) -> Result<Self,Self::Error> {
        if value.is_dir() {
            Ok(HashDirectoryRead { inner:value })
        } else {
            Err(Box::new(GlobalError::InitError(String::from("Value from HashDirectoryRead must be a directory"))))
        }
    }
}

impl<'cow> ReadStrategies for HashDirectoryRead<'cow> {

    fn flush(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(),Box<dyn Error>> {
        let reader = FileReader::try_from(self.inner)?;
        reader.flush_data(buffers)?;
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

impl ReadStrategy {
    pub fn excute_reader_strategy<'a>(&'a self,buffer:&mut Vec<Arc<[u8]>>,path:&'a Path)-> Result<(),Box<dyn Error>>
    {
        let result:Box<dyn ReadStrategies> = match &self {
            Self::Smale => Box::new(SmaleRead::new( &path)),
            Self::Medium => Box::new(MediumRead::new(&path)?),
            Self::Large => Box::new(ChunckRead::new(&path, CHUNK_SMALL_SLICE)),
            Self::ExtraLarge => Box::new(ChunckRead::new(&path, CHUNK_MEDIUM_SLICE)),
        };
        result.flush(buffer)?;
        Ok(())
    }
}

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
pub struct FileReader
{
    inner:Arc<Path>,
    strategy:ReadStrategy
}

impl Deref for FileReader {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
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

    pub fn flush_data(&self,buffers:&mut Vec<Arc<[u8]>>)->Result<(), io::Error>
    {
        self.strategy.excute_reader_strategy(buffers, &self.inner)
        .map_err(|_|io::Error::new(io::ErrorKind::Other, "strategy can't handle reading"))?;
        Ok(())
    }

}


//-------------------------------------------------------------------------
//----------------------------storage-Strategies---------------------------
//-------------------------------------------------------------------------


pub trait StorageStrategies<'path> where Self:AsRef<Path>
{
    fn init_data_storage(&self,buffers:&mut Vec<u8>)->Result<(),Box<dyn Error>> 
    {
        let mut file = match self.as_ref().is_dir() {
            true => {
                fs::create_dir(self)?;
                let mut data_qcow = self.as_ref().to_path_buf();
                data_qcow.push("index.qcow");
                fs::File::create_new(data_qcow)?
            },
            false => {
                fs::File::create_new(self)?
            }
        };
        file.write(buffers)?;
        Ok(())
    }
}

//-------------------------------------------------------------------------
struct NormalFile<'file> { 
    parent: Cow<'file,Path>
}

impl<'file> From<&'file Path> for NormalFile<'file> {
    fn from(value: &'file Path) -> Self {
        NormalFile { parent: Cow::Borrowed(value) }
    }
}

impl<'file> AsRef<Path> for NormalFile<'file> {
    fn as_ref(&self) -> &Path {
        &self.parent
    }
}

impl<'file> From<&'file NormalFile<'file>> for PathBuf {
    fn from(value: &'file NormalFile) -> Self {
        value.into()
    }
}

// impl<'file> StorageStrategies<'file> for NormalFile<'file> {

//     fn store_in(&self,file:fsbuffers:&mut Vec<u8>)->Result<(),Box<dyn Error>> {
        
//         for buffer in buffers.iter() {
//             file.write(buffer)?;
//         }
//         Ok(())
//     }
// }

//-------------------------------------------------------------------------

struct HashContainerFile<'file> {
    parent: HashDirectoryRead<'file>
}

impl<'file> TryFrom<&'file Path> for HashContainerFile<'file> {
    type Error = Box<dyn Error>;

    fn try_from(value: &'file Path) -> Result<Self,Self::Error> {
        Ok( 
            HashContainerFile { parent: HashDirectoryRead::try_from(value)? }
        )
    }
}

impl<'file> From<&'file HashContainerFile<'file>> for PathBuf {
    fn from(value: &'file HashContainerFile) -> Self {
        value.into()
    }
}

impl<'file> AsRef<Path> for HashContainerFile< 'file> {
    fn as_ref(&self) -> &Path {
        self.parent.inner
    }
}

impl<'file> StorageStrategies<'file> for HashContainerFile<'file> {}

//-------------------------------------------------------------------------

// struct StorageNavigator<'a,S:StorageStrategies<'a>>{
//     strategy:&'a S
// }

// impl<'a,S:StorageStrategies<'a>> StorageNavigator<'a,S> {
//     fn new(strategy:&'a S)->Self
//     {
//         // self.strategy = Some(strategy);
//         StorageNavigator { strategy:strategy }
//     }

//     fn storage(self,buffers:&mut Vec<Box<[u8]>>)->Result<(),Box<dyn Error>>
//     {
//         self.strategy.store_in(buffers)?;
//         Ok(())
//     }
// }


pub fn storage_gestion(path:&Path,buffers:&mut Vec<u8>)->Result<(),Box<dyn Error>>
{
    let storage_type:Box<dyn StorageStrategies> = match path.metadata()?.len() {
        x if x <= LARGE_FILE => Box::new(NormalFile::from(path)),
        _ => Box::new(HashContainerFile::try_from(path)?)
    };
    storage_type.store_in(buffers)?;
    Ok(()) 
}



// struct CompositeHashFile<'dir> {
//     inner: Cow<'dir,Path>
// }

// impl<'dir> From<&'dir Path> for CompositeHashFile<'dir> {
//     fn from(value: &'dir Path) -> Self {
//         CompositeHashFile { inner:  Cow::Borrowed(value)}
//     }
// } 


// impl<'dir> StorageStrategies<'dir> for CompositeHashFile<'dir> {

//     fn store_in(&self,buffers:&mut Vec<Box<[u8]>>)->Result<(),Box<dyn Error>> {
//         for buffer in buffers.iter() {
//             let mut temp_path = self.inner.to_path_buf();
//             let hash_stringify = blake3::hash(buffer).to_string();
//             temp_path.extend(&PathBuf::from(hash_stringify));
//             let mut file = fs::File::create_new(temp_path)?;
//             file.write(buffer)?;
//         }
//         Ok(())
//     }
// }