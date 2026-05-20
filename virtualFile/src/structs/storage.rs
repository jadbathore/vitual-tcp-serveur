use std::{
    // borrow::Cow, 
    path::{Path,PathBuf}, pin::Pin, 
    sync::Arc
};
use futures::io;
use tokio::{
    fs::{self,File},
    // task::JoinHandle
}; 

use commun_utils_handler::fs_strategies::LARGE_FILE;


pub trait StorageStrategies where Self:AsRef<Path>
{
    fn init_data_storage<'path>(self:Arc<Self>)->BoxFuture<'path,Result<File,io::Error>>;
}

//-------------------------------------------------------------------------
struct NormalFile { 
    parent: Arc<Path>
}


impl<'path> From<&'path Path> for NormalFile {
    fn from(value: &'path Path) -> Self {
        NormalFile { parent: Arc::from(value) }
    }
}

impl<'path> AsRef<Path> for NormalFile {
    fn as_ref(&self) -> &Path {
        &self.parent
    }
}

impl<'path> From<NormalFile> for PathBuf {
    fn from(value:NormalFile) -> Self {
        value.into()
    }
}

impl<'path> StorageStrategies for NormalFile
{
    fn init_data_storage<'pinbox>(self:Arc<Self>)->BoxFuture<'pinbox,Result<File,io::Error>>
    {
        Box::pin(async move {
            File::create_new(self.as_ref()).await
        })
    }
}

//-------------------------------------------------------------------------

struct HashContainerFile {
    parent: Arc<Path>
}

// impl<'file> HashContainerFile<'file> {
//     async fn file_creation(&self)->Result<File, std::io::Error>
//     {
//         fs::create_dir(self).await?;
//         let mut data_qcow = self.as_ref().to_path_buf();
//         data_qcow.push("index.qcow");
//         File::create_new(data_qcow).await
//     }
// }
impl<'path> From<&'path Path> for HashContainerFile {
    fn from(value: &'path Path) -> Self{
        HashContainerFile { parent: Arc::from(value) }
    }
}

impl<'file> From<HashContainerFile> for PathBuf {
    fn from(value: HashContainerFile) -> Self {
        value.into()
    }
}

impl<'file> AsRef<Path> for HashContainerFile {
    fn as_ref(&self) -> &Path {
        &self.parent
    }
}

impl<'path> StorageStrategies for HashContainerFile {
    fn init_data_storage<'pinbox>(self:Arc<Self>)->BoxFuture<'pinbox,Result<File,io::Error>>
    {
        Box::pin(async move {
            fs::create_dir(self.as_ref()).await?;
            let mut data_qcow = self.as_ref().as_ref().to_path_buf();
            data_qcow.push("index.qcow");
            File::create_new(data_qcow).await
        })
    }
}

fn get_dyn_storage_strategy<'path>(path:&'path Path,predicate:usize)->Arc<dyn StorageStrategies + 'path> 
{
    match predicate {
        x if x <= LARGE_FILE as usize => Arc::from(NormalFile::from(path)),
        _ => Arc::from(HashContainerFile::from(path))
    }
}

pub async fn storage_strategy<'path>(path:&'path Path,predicate:usize)->Result<File, std::io::Error>
{
    let storage_type:Arc<dyn StorageStrategies> = get_dyn_storage_strategy(path, predicate);
    storage_type.init_data_storage().await 
}
