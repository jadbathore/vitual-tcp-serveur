#[allow(warnings)]
mod bindings;

mod utils;
use bindings::Guest;
use regex::Regex;
use commun_utils_handler::{FileScanner, ScanBytesSubject, fs_strategies::{FileReader, recursive_file_read}};
use std::path::Path;
// use commun_utils_handler::FileScanner;
use crate::utils::lexer::{MalwareWarnRaiseApp, MalwareWarnRaiseImg};
use indicatif::ProgressBar;
struct Component;
use boa_engine::{Context, Source};


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


fn scan(files:Vec<FileReader<&Path>>,scanner:&mut ScanBytesSubject)
{
    let progress_bar_application = ProgressBar::new(files.len() as u64);
    for file in files {
        scanner.scan_data(file).unwrap();
        progress_bar_application.inc(1);
    }
    progress_bar_application.finish();
}



impl Guest for Component {

    fn ta0043()
    {
        let mut image_files:Vec<FileReader<&Path>> = Vec::new();
        let mut application_files:Vec<FileReader<&Path>> = Vec::new();
        recursive_file_read(Path::new("./fs"), &mut |path| {
            if Regex::new(r"(?i)\.(((c|m)?js)|wasm)").unwrap().is_match(&path.to_string_lossy()) {
                application_files.push(FileReader::try_from(path).unwrap());
            }   
            if Regex::new(r"(?i).(jpe?g|png)").unwrap().is_match(&path.to_string_lossy()){
                image_files.push(FileReader::try_from(path).unwrap());
            }
            Ok(())
        }).unwrap();
        
        scan(image_files, &mut MalwareWarnRaiseImg::scanner());
        scan(application_files,&mut  MalwareWarnRaiseApp::scanner());
    }

    fn exec_utils(js_code:String)-> String {
        let mut context = Context::default();

        // Parse the source code
        match context.eval(Source::from_bytes(&js_code)) {
            Ok(res) => {
                res.to_string(&mut context).unwrap().to_std_string_escaped()
            }
            Err(e) => {
                // Pretty print the error
                eprintln!("Uncaught {e}");
                String::new()
            }
        }
    }
}

bindings::export!(Component with_types_in bindings);
