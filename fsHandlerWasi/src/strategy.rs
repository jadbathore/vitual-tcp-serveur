use std::{
    borrow::Cow, 
    error::Error, 
    fs::{self,File}, 
    io::{self,BufReader,Read, Stdout}, path::Path
};

use crate::{
    enums::errors::GlobalHandlerError
};

pub const MEDIUM_FILE: u64 = 64 * 1024;
pub const LARGE_FILE: u64 = 10 * 1024 * 1024;
pub const HUGE_FILE: u64 = 100 * 1024 * 1024;
pub const GIGA_FILE: u64 = 1 * 1024 * 1024 * 1024; 
pub const CHUNK_SMALL_SLICE:u64 = 128 * 1024;
pub const CHUNK_SMALL_MEDIUM:u64 =  CHUNK_SMALL_SLICE * 2;


pub trait ReadStrategies {
    // type Buffer;
    fn flush(&self,buffers:&mut Vec<Vec<u8>>)->Result<(),Box<dyn Error>>;
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
    fn flush(&self,buffers:&mut Vec<Vec<u8>>)->Result<(),Box<dyn Error>>
    {
        let data = fs::read(&self.inner)?;
        buffers.push(data);
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
                    GlobalHandlerError::from(err)
        })?;
        Ok( MediumRead { inner: Cow::from(entry) ,capacity:capacity } )
    }
}

impl<'cow> ReadStrategies for MediumRead<'cow> {
    // type Data = Vec<u8>;
    fn flush(&self,buffers:&mut Vec<Vec<u8>>)->Result<(),Box< dyn Error>> {
        let data = fs::File::open(&self.inner)?;
        let mut sub_buf:Vec<u8> = Vec::with_capacity(self.capacity); 
        let mut reader = BufReader::new(data);
        reader.read_to_end(&mut sub_buf)?;
        buffers.push(sub_buf);
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
        // let data = fs::File::open(entry)?;
        ChunckRead {inner: Cow::from(entry), chunck_size:chunk_size}
    }
}

// impl<'a> TryFrom<&'a Path> for ChunckRead {
//     type Error = Box<dyn Error>;
//     fn try_from(entry: &'a Path) -> Result<Self, Self::Error> {
//         let data = fs::File::open(entry)?;
//         Ok( MediumRead { file: data ,capacity: } )
//     }
// }

impl<'cow> ReadStrategies for ChunckRead<'cow> {
    // type Data = Vec<u8>;
    fn flush(&self,buffers:&mut Vec<Vec<u8>>)->Result<(),Box< dyn Error>> {
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
        buffers.push(sub_capacity_buffer);
        Ok(())
    }
}


struct StreamRead {
    stream:Stdout
}


struct ReadNavigator<T:ReadStrategies> where  
    T:
{
    strategies:T
}

impl<T:ReadStrategies> ReadNavigator<T> 
{
    pub fn new(strategy:T)->Self {
        ReadNavigator { strategies: strategy }
    }

    fn read(self,buffer:&mut Vec<Vec<u8>>){
        self.strategies.flush(buffer);
    }

    
}

struct ReadEntry<'path> {
    path:Cow<'path,Path>,
    strategy:ReadStrategy
}



impl<'path> ReadEntry<'path>  {
    fn new(path:Cow<'path,Path>)-> Result<Self,Box<dyn Error>>
    {
        let read = ReadStrategy::try_from(path.as_ref())?;
        Ok(
            ReadEntry { 
                path:path, 
                strategy: read
            }
        )
    }


}

#[derive(Debug)]
pub enum ReadStrategy {
    Smale,
    Medium,
    Large,
    ExtraLarge,
    GigaLarge
}

impl<'buffer> TryFrom<&'buffer Path> for ReadStrategy {
    type Error = Box<dyn Error>;
    fn try_from(entry: &'buffer Path) -> Result<Self,Self::Error> {
        match entry.metadata()?.len() { 
            x if x <= MEDIUM_FILE => Ok(ReadStrategy::Smale),
            x if x <= LARGE_FILE => Ok(ReadStrategy::Medium),
            x if x <= HUGE_FILE => Ok(ReadStrategy::Large),
            x if x <= GIGA_FILE  => Ok(ReadStrategy::ExtraLarge),
            _ => Ok(ReadStrategy::GigaLarge)
        }
    }
}
// trait BoxedStrategy<'a> {
//     fn as_ref(&'a self) -> &dyn ReadStrategies;
// }

// impl<'a> BoxedStrategy<'a> for Box<dyn ReadStrategies> {
//     fn as_ref(&'a self) -> &dyn ReadStrategies {
//         AsRef::as_ref(self)
//     }
// }

impl ReadStrategy {
    pub fn excute_reader_strategy<'a>(&'a self,buffer:&mut Vec<Vec<u8>>,path:&'a Path)-> Result<(),Box<dyn Error>>
    {
        let result:Box<dyn ReadStrategies> = match &self {
            Self::Smale => Box::new(SmaleRead::new( &path)),
            Self::Medium => Box::new(MediumRead::new(&path)?),
            Self::Large => Box::new(ChunckRead::new(&path, 128 * 1024)),
            Self::ExtraLarge => Box::new(ChunckRead::new(&path, 256 * 1024)),
            Self::GigaLarge => todo!()
        };
        result.flush(buffer)?;
        
        // BoxedStrategy::as_ref(&result).flush(buffer);
        //  = result.as_ref();
        // let nav = ReadNavigator::new(result);
        Ok(())
    }
}

// impl ReadStrategy {
//     fn excute<'path,T:ReadStrategies + TryFrom<Cow<'static,Path>>>(&self,entry:Cow<'path,Path>)->Result<ReadNavigator<T>,Box<dyn Error>>
//     {
//         match self {
//             ReadStrategy::Smale => {
//                 ReadNavigator::new(SmaleRead::try_from(entry)?)
//             }
//         }
//     }
// }
// impl<'buffer> ReadStrategy<'buffer> {
//     pub fn from<'a>(entry:Cow<'a,Path>)->Result<Self, Box<dyn Error>>
//     {
//         match entry.metadata()?.len() {
//             x if x <= MEDIUM_FILE => {
//                 let a = fs::read(entry)?;
//                 Ok(ReadStrategy::Smale(a))
//             },
//             x if x <= LARGE_FILE => {
//                 let file = fs::File::open(entry)?;
//                 let capacity = <u64 as TryInto<usize>>::try_into(x).map_err(|err|{
//                     GlobalHandlerError::from(err)
//                 })?;
//                 let mut buf = BufReader::new(file);
//                 Ok(ReadStrategy::Medium(&mut buf ,capacity))
//             },
//             x if x <= HUGE_FILE => {
//                 let file = File::open(entry)?;
//                 let capacity:usize = 128 * 1024;
//                 let mut reader = BufReader::with_capacity(capacity, file);
//                 // let buffer:Vec<u8> = Vec::with_capacity(128*1024);
//                 // let a = reader.read_exact(&mut buffer)?;
//                 Ok(ReadStrategy::Large(&mut reader,capacity))
//             },
//             x if x <= GIGA_FILE  => {
//                 let file = File::open(entry)?;
//                 let capacity:usize = 256 * 1024;
//                 let reader = BufReader::with_capacity(capacity, file); 
//                 Ok(ReadStrategy::ExtraLarge(&mut reader,capacity))
//             },
//             _ => {
//                 let stdout = io::stdout();
//                 if stdout.is_terminal()
//                 {
//                     let file = File::open(entry)?;
//                     let reader = BufReader::new(file);
//                     Ok(ReadStrategy::GigaLarge(stdout,reader))
//                 } else {
//                     Err(Box::new(GlobalHandlerError::NotTerminal))
//                 }
//             }
//         }
//     }
// }
// // // pub fn format_file_size<'a>(entry: &DirEntry) -> Result<SizeFile<'a>, Box<dyn Error>> 
// // // {
// // //     const MEDIUM_FILE: u64 = 64 * 1024;
// // //     const LARGE_FILE: u64 = 10 * 1024 * 1024;
// // //     const HUGE_FILE: u64 = 100 * 1024 * 1024;
// // //     const GIGA_FILE: u64 = 1 * 1024 * 1024 * 1024; 

// // //     dbg!(entry.metadata()?.len());
// // //     match entry.metadata()?.len() {
// // //         x if x <= MEDIUM_FILE => {
// // //             let a = fs::read_to_string(entry.path())?;
// // //             Ok(SizeFile::Smale(a.into()))
// // //         },
// // //         x if x <= LARGE_FILE => {
// // //             let string = String::with_capacity(x as usize);
// // //             let file = fs::File::open(entry.path())?;
// // //             Ok(SizeFile::Medium(BufReader::new(file)))
// // //         },
// // //         x if x <= HUGE_FILE => {
// // //             let file = File::open(entry.path())?;
// // //             let mut reader = BufReader::with_capacity(128 * 1024, file);
// // //             let mut buffer:Vec<u8> = Vec::with_capacity(128*1024);
// // //             reader.read_until(b'\n', &mut buffer)?;
// // //             Ok(SizeFile::Large(reader,buffer))
// // //         },
// // //         x if x <= GIGA_FILE  => {
// // //             let file = File::open(entry.path())?;
// // //             let mut reader = BufReader::with_capacity(256 * 1024, file); 
// // //             let mut buffer:Vec<u8> = Vec::with_capacity(256*1024);
// // //             reader.read_until(b'\n', &mut buffer)?;
// // //             Ok(SizeFile::ExtraLarge(reader,buffer))
// // //         },
// // //         _ => {
// // //             let file = File::open(entry.path())?;
// // //             let reader = BufReader::new(file);
// //             let buffer:Vec<u8> = Vec::with_capacity(512*1024); 
// //             let stdout = test_std()?;
// //             Ok(SizeFile::GigaLarge(stdout,reader,buffer))
// //         }
//     }
// }



// impl SizeFile
// {
//     // fn handler(reader:&mut BufReader<File>,buffer:&mut Vec<u8>)->Result<(),Box<dyn Error>>
//     // {
//     //     loop {
//     //         let bytes_read = reader.read(buffer)?;
//     //         if bytes_read == 0 {
//     //             break;
//     //         }
//     //     };
//     //     Ok(())
//     // }

//     // pub fn read(self)->Result<(),Box< dyn Error>>
//     // {

//     //     match self {
//     //         SizeFile::Smale(content) => {
//     //             content;
//     //         },
//     //         SizeFile::Medium(mut buffer) => {
//     //             let mut line = String::new();
//     //             // while buffer.read_line(&mut line)? > 0 {
//     //             //     line.clear(); // Réutilisation du buffer
//     //             // }
//     //         },
//     //         SizeFile::Large(mut reader,mut buffer)|SizeFile::ExtraLarge(mut reader,mut buffer) => {
//     //             Self::handler(&mut reader, &mut buffer);
//     //         },
//     //         SizeFile::GigaLarge(stdout,reader,buffer) => {
//     //             let mut handle = stdout.lock();
//     //             // loop {
//     //             //     let bytes_read = reader.read(buffer)?;
//     //             //     if bytes_read == 0 {
//     //             //         break;
//     //             //     }
//     //             // }
//     //             // handle.write_all(buffer);
//     //         }
//     //     };
//     //     Ok(())
//     // }
// }
