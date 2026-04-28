use std::{error::Error, fs::write, path::Path};

use blake3::Hasher;
use commun_utils_handler::fs_strategies::{FileReader, recursive_file_read};

fn main()->Result<(),Box<dyn Error>>{
    
    let mut files:Vec<FileReader> = Vec::new();
    recursive_file_read(Path::new("../test"),&mut |i|{
        files.push(FileReader::try_from(i)?);
        Ok(())
    })?;

    let mut hasher = Hasher::new();
    for file in files.iter() {
        let mut buffers = Vec::new();
        file.flush_data(&mut buffers)?;
        hasher.update(file.get_string_lossy_url().as_bytes());
        for buffer in buffers.iter()
        {
            hasher.update(buffer);
        }
    }
    let hash = hasher.finalize();

    write("../snapshot.bin",hash.to_hex().as_bytes())?;
    Ok(())
}
