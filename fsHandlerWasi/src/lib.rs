#[allow(warnings)]
mod bindings;

mod utils;
pub mod commun_utils;
use bindings::Guest;
use regex::Regex;
use std::{collections, error::Error, path::{Path, PathBuf}};

use crate::{
    commun_utils::{item::FileReader, read_strategies::{self, ReadStrategy, recursive_file_read}}, 
    utils::payload_collection::{Collection, DataIterator, DataCollection}
};


struct Component;




// fn build_preopendir_collection(filter:bool)->Result<ReadCollection<(PathBuf,ReadStrategy)>,Box<dyn Error>>
// {
//     let mut paths:Vec<(PathBuf,ReadStrategy)> = Vec::new();
//     recursive_file_read(Path::new("./fs"), &mut |path| {
//         if filter {
//             let read = ReadStrategy::try_from(path)?; 
//             paths.push((path.into(),read));
//         }
//         Ok(())
//     }).unwrap();
//     Ok(ReadCollection::from(paths))
// }



impl Guest for Component {

    // test d'observation des fichiers et comportement 
    fn ta0043()
    {
        let mut paths:Vec<FileReader> = Vec::new();
        recursive_file_read(Path::new("./fs"), &mut |path| {
            let app_regex = Regex::new(r"(?i)\.(((c|m)?js)|wasm)").unwrap();
            if app_regex.is_match(&path.to_string_lossy()) {
                paths.push(FileReader::new(path).unwrap());
            }   
            Ok(())
        }).unwrap();
        let test = DataCollection::from(paths);
        
    }

}

bindings::export!(Component with_types_in bindings);
