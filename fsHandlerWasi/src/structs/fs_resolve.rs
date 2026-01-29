use std::{
    error::Error, ffi::OsString, fs::{self,DirEntry, File}, io::{self, BufReader, IsTerminal, Stdout}, path::PathBuf, rc::Rc
};
use std::io::BufRead;
use crate::traits::fs_read::{DirReader, FileReader};
use std::io::Read;
pub struct Resolve;
 

impl Resolve {
    fn get_entries(path:&PathBuf)-> Result<Vec<DirEntry>,io::Error>
    {
        match fs::read_dir(path) {
            Ok(dir) => {
                dir.collect::<Result<Vec<DirEntry>, io::Error>>()
            },
            Err(err) => Err(err)
        }
    }
}


impl FileReader for Resolve {
    fn recursive<'a>(path:&PathBuf,entries:&mut Vec<String>)->Result<(), io::Error>
    {
        for entry in Resolve::get_entries(&path)?.into_iter() {
            if entry.file_type()?.is_file() {
                // let path = entry.path().as_os_str().to_str();
                // let cow =  path.as_os_str().to_string_lossy();
                if let Some(str) = entry.path().as_os_str().to_str()
                {
                    entries.push(str.to_string());
                } else  {
                    return  Err(io::Error::new(io::ErrorKind::InvalidFilename, "file name not readable"));
                }
            
            } else {
                let mut clone_sub = path.clone();
                let sub_buf = PathBuf::from(entry.file_name());
                clone_sub.push(sub_buf);
                let value = <Resolve as DirReader>::recursive(clone_sub)?;
                entries.extend(value);
            }   
        }
        Ok(())
    }
}

impl DirReader for Resolve {
    fn recursive(path:PathBuf)->Result<Vec<String>, io::Error>
    {
        let mut directories:Vec<String> = Vec::new();
        <Resolve as FileReader>::recursive(&path,&mut directories)?;
        Ok(directories)
    }
}


