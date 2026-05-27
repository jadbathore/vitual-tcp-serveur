use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FakeToSubPath {
    inner:PathBuf,
    fake:Option<PathBuf>
}


impl<'path> AsRef<Path> for FakeToSubPath {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl <'path> From<&'path Path> for Box<FakeToSubPath> {
    fn from(value: &'path Path) -> Self {
        Box::new(FakeToSubPath { inner: value.to_path_buf(), fake: None })
    }
}

impl From<PathBuf> for Box<FakeToSubPath> {
    fn from(value: PathBuf) -> Self {   
        let mut sub  = value.clone();
        sub.pop();
        Box::new(FakeToSubPath { inner: value , fake: Some(sub) })
    }
}

impl FakeToSubPath {

    pub fn get_link<'path>(&'path self)->&'path Path 
    {
        if let Some(fake) = &self.fake {
            &fake
        } else  {
            &self.inner
        }
    }
}