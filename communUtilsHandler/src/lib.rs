pub mod errors;
pub mod fs_strategies;
pub mod collection;

use std::{borrow::Cow, collections::{HashMap, HashSet}, error::Error, iter, sync::Arc};

use regex::bytes::{Regex, RegexSet};

use crate::{collection::{Collection, GenericCollection}, fs_strategies::FileReader};

pub trait FileScanner {
    fn scanner<'scanner>()->ScanBytesSubject<'scanner>;
    // fn test();
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
    regexes:Vec<ScanWarnByte<'keys>>
}

// impl<'keys> TryFrom<Vec<(&'keys str,String)>> for ScanBytesSubject<'keys>  
// {
//     type Error = Box<dyn Error>;

//     fn try_from(regexes: Vec<(&'keys str,String)>) -> Result<Self, Self::Error> 
//     {

//         // let regex = regexes.;
//         // let regex_value:Vec<String> = regexes.iter().map(|a|a.1).collect();

//         let a = regexes.iter().map(|(key,value)|{
//             ScanWarnByte::new(*key, Regex::new(value).unwrap())
//             // (,Regex::new(value).unwrap())
//         }).collect();
        
//         Ok(ScanBytesSubject { regex_set: regex_set, regexes:a})
//     }
// }

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

    pub fn scan_data<'file>(&self,file:FileReader)->Result<(),Box<dyn Error>>
    {
        let mut buffers:Vec<Arc<[u8]>> = Vec::new();
        file.flush_data(&mut buffers)?;
        // let binder:Vec<&ScanWarnByte> = Vec::new();
        // let mut collection = GenericCollection::from(binder);
        let mut set:HashSet<usize> = HashSet::new();

        for data in buffers {
            set.extend(self.regex_set.matches(&data).iter().collect::<Vec<usize>>());
        }

        let warn:Vec<&str> = set.iter().map(|i|{
            self.regexes[*i].get_warn()
        }).collect();
        if !warn.is_empty() {
            println!("found on file {} :\n\t-{}",file.get_string_lossy_url(),warn.join("\n\t-"));
            
        }
        Ok(())
        // let matches = self.regex_set.matches(data);
        // for i in matches.iter() {
        //     let current_regex = &self.regexes[i];
        //     if current_regex.get_byte_regex().is_match(data) {
        //         println!("warn {} found on file {}",current_regex.get_warn(),file_name);
        //     }
        // }
    }
}


fn tes<'a>(a:usize)->ScanBytesSubject<'a>
{
    const A:usize = 1000;
    let b:[&str;A] = ["";A];
    let c = ["";A];
    ScanBytesSubject::new::<A>(b,c).unwrap()
}
// #[derive(Debug)]
// pub struct ScanWarn<'haystack> {
//     regex:Regex,
//     name:&'haystack str,
// }

// impl<'haystack> ScanWarn<'haystack> {

//     pub fn new(regex:Regex,name:&'haystack str)->Self
//     {
//         ScanWarn { 
//             regex,
//             name,
//         }
//     }

//     pub fn get_name(&self)->String
//     {
//         self.name.to_string()
//     }

//     pub fn threat_score<'a>(&mut self,content:&[u8])-> usize
//     {
        
//         if let Some(capture) = self.regex.captures(content) {
//             capture.iter().len()
//         } else {
//             0
//         }

//         // self.regex.captures(content.into()).iter().len();
//     }

// }




