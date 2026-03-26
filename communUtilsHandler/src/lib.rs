pub mod errors;
pub mod fs_strategies;
pub mod collection;

use std::{borrow::Cow, collections::{HashMap, HashSet}, error::Error, iter, sync::Arc};

use regex::bytes::{Regex, RegexSet};

use crate::{collection::{Collection, GenericCollection}, fs_strategies::FileReader};


static CAP_ERROR: usize = 10;

pub trait FileScanner {
    fn scanner<'scanner>()->ScanBytesSubject<'scanner>;
}

pub struct ScanWarnByte<'key> {
    warn_name:&'key str,
    regex:Regex
}

impl<'key> ScanWarnByte<'key> {

    fn get_warn(&self)->&'key str 
    {
        self.warn_name
    }

    fn get_byte_regex(&self)->&Regex
    {
        &self.regex
    }
}



pub struct ScanBytesSubject<'keys> {
    regex_set:RegexSet,
    regexes:Vec<ScanWarnByte<'keys>>,
}

impl<'keys> ScanBytesSubject<'keys> 
{
    pub fn new<const N: usize>(warn: [&'keys str; N], regex_name: [&'keys str; N])->Result<Self, Box<dyn Error>> 
    {
        let regex_set = RegexSet::new(regex_name)?;
        let regexes = warn.iter().enumerate().map(|( key,value)|{
            ScanWarnByte { warn_name:value, regex: Regex::new(regex_name[key]).unwrap() }
        }).collect();
        Ok(ScanBytesSubject { regex_set: regex_set, regexes:regexes})
    }

    pub fn scan_data<'file>(&mut self,file:FileReader)->Result<(),Box<dyn Error>>
    {
        let mut buffers:Vec<Arc<[u8]>> = Vec::new();
        file.flush_data(&mut buffers)?;
        // let binder:Vec<&ScanWarnByte> = Vec::new();
        // let mut collection = GenericCollection::from(binder);
        let mut set:HashSet<usize> = HashSet::new();
        let mut warn_score:usize = 0;
        for data in buffers {
            for index in  self.regex_set.matches(&data).iter() {
                if !set.contains(&index) {  
                    warn_score += self.regexes[index].get_byte_regex().captures(&data).iter().len();
                    set.insert(index);
                }
            }
            // set.extend(self.regex_set.matches(&data).iter().collect::<Vec<usize>>());

        }

        let warn:Vec<&str> = set.iter().map(|i|{
            self.regexes[*i].get_warn()
        }).collect();

        if !warn.is_empty() {
            if CAP_ERROR < warn_score {
                panic!("A suspicious file '{}' containing too many dangerous elements\nprevents the program from functioning:\n\t-{}",file.get_string_lossy_url(),warn.join("\n\t-"))
            }
            println!("found on file {} :\n\t-{}\nWith a score of warn_score {} ",file.get_string_lossy_url(),warn.join("\n\t-"),warn_score);
        }
        Ok(())
    }

}

