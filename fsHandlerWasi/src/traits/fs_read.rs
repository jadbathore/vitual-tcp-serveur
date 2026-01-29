use std::{
    fs::{self,DirEntry},
    io, path::PathBuf,
};

pub trait FileReader {
    fn recursive(path:&PathBuf,entries:&mut Vec<String>)->Result<(), io::Error>;
}

pub trait DirReader {
    fn recursive(path:PathBuf)->Result<Vec<String>, io::Error>;
}

