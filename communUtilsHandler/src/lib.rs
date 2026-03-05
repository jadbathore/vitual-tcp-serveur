use regex::{Captures, Regex};
pub trait FileScanner {
    fn scan(content:String);
}

#[derive(Debug)]
pub struct ScanWarn<'haystack> {
    regex:Regex,
    name:&'haystack str,
}

impl<'haystack> ScanWarn<'haystack> {

    pub fn new(regex:Regex,name:&'haystack str)->Self
    {
        ScanWarn { 
            regex,
            name,
        }
    }

    pub fn get_name(&self)->String
    {
        self.name.to_string()
    }

    pub fn threat_score<'a>(&mut self,content:&'a str)-> usize
    {
        if let Some(capture) = self.regex.captures(content) {
            capture.iter().len()
        } else {
            0
        }

        // self.regex.captures(content.into()).iter().len();
    }

}