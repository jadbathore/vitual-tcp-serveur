#[allow(warnings)]
mod bindings;

mod utils;
use bindings::Guest;
use regex::Regex;
use commun_utils_handler::{FileScanner, ScanBytesSubject, fs_strategies::{FileReader, recursive_file_read}};
use std::{collections, error::Error, path::{Path, PathBuf}, string::FromUtf8Error, sync::Arc};
// use commun_utils_handler::FileScanner;
use crate::utils::lexer::{MalwareWarnRaiseApp, MalwareWarnRaiseImg};
use indicatif::ProgressBar;
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


fn scan(files:Vec<FileReader>,scanner:ScanBytesSubject)
{
    let progress_bar_application = ProgressBar::new(files.len() as u64);
    for file in files.iter() {
        let mut buffers:Vec<Arc<[u8]>> = Vec::new();
        file.flush_data(&mut buffers).unwrap();
        for data in buffers {
            scanner.scan_data(&data,file.get_string_lossy_url());
        }
        progress_bar_application.inc(1);
    }
    progress_bar_application.finish();
}



impl Guest for Component {

    // test d'observation des fichiers et comportement 
    fn ta0043()
    {
        let mut image_files:Vec<FileReader> = Vec::new();
        let mut application_files:Vec<FileReader> = Vec::new();
        recursive_file_read(Path::new("./fs"), &mut |path| {
            if Regex::new(r"(?i)\.(((c|m)?js)|wasm)").unwrap().is_match(&path.to_string_lossy()) {
                application_files.push(FileReader::new(path).unwrap());
            }   
            if Regex::new(r"(?i).(jpe?g|png)").unwrap().is_match(&path.to_string_lossy()){
                image_files.push(FileReader::new(path).unwrap());
            }
            Ok(())
        }).unwrap();
        
        scan(image_files, MalwareWarnRaiseImg::scanner());
        scan(application_files, MalwareWarnRaiseApp::scanner());
        // let mut paths:Vec<FileReader> = Vec::new();

        // let regex = Regex::new(r"https").unwrap();
        // dbg!(regex.captures("https").iter().len());
        // MalwareWarnRaise::scan(String::from("https://"));
        // let malware_raise = MalwareWarnRaise::new();
        // MalwareWarnRaise::scan(String::from("https"));
        // println!("test");
        // test();
        // let mut paths:Vec<FileReader> = Vec::new();
        // recursive_file_read(Path::new("./fs"), &mut |path| {
        //     let app_regex = Regex::new(r"(?i)\.(((c|m)?js)|wasm)").unwrap();
        //     if app_regex.is_match(&path.to_string_lossy()) {
        //         paths.push(FileReader::new(path).unwrap());
        //     }   
        //     Ok(())
        // }).unwrap();
        // let test = DataCollection::from(paths);
        
    }

}

bindings::export!(Component with_types_in bindings);
