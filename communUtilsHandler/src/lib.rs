use regex::Regex;
pub trait FileScanner {
    fn scan();
}

struct ScanWarn {
    regex:Regex,
    name:String,
    score:u64
}




impl ScanWarn {

    fn new(regex:Regex,name:String)->Self
    {
        ScanWarn { 
            regex,
            name,
            score:0 
        }
    }

    fn scan<F>(&self,content:F)->bool
        where
            F:AsRef<[u8]>,
            for<'a> &'a str: From<F>
    {
        self.regex.is_match(content.into())
        // let a:String = content.into();
    }

}