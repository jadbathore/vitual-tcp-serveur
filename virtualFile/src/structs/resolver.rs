use std::{error::Error,fs::{self,DirEntry}, path::Path,io};
use crate::{predicate_cache_use, structs::payloads::payload::DataFile};


fn get_entries(path:&Path)-> Result<Vec<DirEntry>,io::Error>
{
    match fs::read_dir(path) {
        Ok(dir) => {
            dir.collect::<Result<Vec<DirEntry>, io::Error>>()
        },
        Err(err) => Err(err)
    }
}

pub fn recusive_dispacher<'a>(path:&Path,to_payload:&mut Vec<DataFile>,to_cache:&mut Vec<DataFile>)->Result<(), Box<dyn Error>> 
{
    for entry in get_entries(path)?.iter() {
        if entry.file_type()?.is_file() {
            let datafile = DataFile::new(entry.path().leak())?;
            if predicate_cache_use(&datafile) {
                to_cache.push(datafile);
            } else {
                to_payload.push(datafile);
            }
        } else {
            recusive_dispacher(entry.path().leak(), to_payload,to_cache)?;
        }
    }
    Ok(())
}








// pub fn recusive<'a>(path:& Path,entries:&mut Vec<DataFile<'_>>)->Result<(), Box<dyn Error>> 
// {
//     let mut  path_buf = PathBuf::from(path);
//     for entry in Resolve::get_entries(path)?.iter()
//     {
//         if entry.file_type()?.is_file() {
//             let datafile = DataFile::new(path)?;
//             entries.push(datafile);
//         } else {
//             recusive(&entry.path(),entries)?;
//         }
//     }
//     Ok(())
// }

// impl FileReader for Resolve {
//     fn recursive<'a,T,I>(path:&'a PathBuf,entries:&'a mut Vec<T>,init:&'a I)->Result<(), Box<dyn Error>> 
//     where 
//         T: PathResolvable,  
//         I: Fn(DirEntry)->Result<T,Box<dyn Error>>
//     {
//         for entry in Resolve::get_entries(&path)?.into_iter() {
//             if entry.file_type()?.is_file() {
//                 let data:T = init(entry)?;
//                 entries.push(data);
//             } else {
//                 let mut clone_sub = path.clone();
//                 let sub_buf = PathBuf::from(entry.file_name());
//                 clone_sub.push(sub_buf);
//                 let value = <Resolve as DirReader>::recursive(&clone_sub,&init)?;
//                 entries.extend(value);
//             }   
//         }
//         Ok(())
//     }
// }



// impl DirReader for Resolve { 
//     fn recursive<'a,T,I>(path:&'a PathBuf,init:&'a I)->Result<Vec<T>, io::Error>
//     where 
//             T: PathResolvable,
//             I: Fn(DirEntry)->Result<T,Box<dyn Error>>,
//     {
//         let mut directories:Vec<T> = Vec::new();
//         <Resolve as FileReader>::recursive(path,&mut directories,&|a|{
//             init(a)
//         });
//         Ok(directories)
//     }
// }


