use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct FakePath {
    inner:PathBuf,
    fake:Option<PathBuf>
}


impl<'path> AsRef<Path> for FakePath {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl <'path> From<&'path Path> for Box<FakePath> {
    fn from(value: &'path Path) -> Self {
        Box::new(FakePath { inner: value.to_path_buf(), fake: None })
    }
}

// impl<'path> From<&'path Path> for FakePath {
//     fn from(value: &'path Path) -> Self {
//         FakePath { inner: value.to_path_buf(), fake: None }
//     }
// }

impl FakePath {

    pub fn set_fake(&mut self,path:PathBuf){
        self.fake = Some(path);
    }

    pub fn get_link<'path>(&'path self)->&'path Path 
    {
        if let Some(fake) = &self.fake {
            &fake
        } else  {
            &self.inner
        }
    }
}