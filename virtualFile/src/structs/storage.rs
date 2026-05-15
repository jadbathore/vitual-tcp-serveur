use std::{
    borrow::Cow, 
    error::Error, 
    path::{Path,PathBuf}, pin::Pin
};


use futures::io;
use tokio::{fs::{self,File}}; 

use commun_utils_handler::fs_strategies::LARGE_FILE;


type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait StorageStrategies<'box_async,'path> where Self:AsRef<Path>, 'path:'box_async
{
    fn init_data_storage(&'path self)->BoxFuture<'box_async,Result<File,io::Error>>;
}

//-------------------------------------------------------------------------
struct NormalFile<'path> { 
    parent: Cow<'path,Path>
}

impl<'path> NormalFile<'path> {
    async fn file_creation(&self)->Result<File, std::io::Error>
    {
        File::create_new(self).await
    }
}

impl<'path> From<&'path Path> for NormalFile<'path> {
    fn from(value: &'path Path) -> Self {
        NormalFile { parent: Cow::Borrowed(value) }
    }
}

impl<'path> AsRef<Path> for NormalFile<'path> {
    fn as_ref(&self) -> &Path {
        &self.parent
    }
}

impl<'path> From<&'path NormalFile<'path>> for PathBuf {
    fn from(value: &'path NormalFile) -> Self {
        value.into()
    }
}

impl<'file,'path> StorageStrategies<'file,'path> for NormalFile<'file> where 'path:'file
{
    fn init_data_storage(&'path self)->BoxFuture<'file,Result<File,io::Error>>
    {
        Box::pin(self.file_creation())
    }
}

//-------------------------------------------------------------------------

struct HashContainerFile<'file> {
    parent: Cow<'file,Path>
}

impl<'file> HashContainerFile<'file> {
    async fn file_creation(&self)->Result<File, std::io::Error>
    {
        fs::create_dir(self).await?;
        let mut data_qcow = self.as_ref().to_path_buf();
        data_qcow.push("index.qcow");
        File::create_new(data_qcow).await
    }
}
impl<'file> TryFrom<&'file Path> for HashContainerFile<'file> {
    type Error = Box<dyn Error>;

    fn try_from(value: &'file Path) -> Result<Self,Self::Error> {
        Ok( 
            HashContainerFile { parent: Cow::Borrowed(value) }
        )
    }
}

impl<'file> From<&'file HashContainerFile<'file>> for PathBuf {
    fn from(value: &'file HashContainerFile) -> Self {
        value.into()
    }
}

impl<'file> AsRef<Path> for HashContainerFile< 'file> {
    fn as_ref(&self) -> &Path {
        &self.parent
    }
}

impl<'file,'path> StorageStrategies<'file,'path> for HashContainerFile<'file> where 'path:'file {
    fn init_data_storage(&'path self)->BoxFuture<'file,Result<File,io::Error>>
    {
        Box::pin(self.file_creation())
    }
}

async fn storage_file<'file,'path>(path:&'path Path,predicate:usize)->Result<File,Box<dyn Error>> where 'path:'file 
{
    let storage_type:Box<dyn StorageStrategies> = match predicate {
        x if x <= LARGE_FILE as usize => Box::new(NormalFile::from(path)),
        _ => Box::new(HashContainerFile::try_from(path)?)
    };
    Ok(storage_type.init_data_storage().await?)

    // Ok(storage_type.init_data_storage()?) 
}
