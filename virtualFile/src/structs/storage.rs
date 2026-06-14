use std::{
    path::{Path,PathBuf}, 
    sync::Arc
};
use blake3::Hash;
use futures::io;
use tokio::{fs::{self,File}, io::AsyncWriteExt}; 
use commun_utils_handler::fs_strategies::LARGE_FILE;

use crate::general::BoxFuture;

pub struct HashData {
    data:Vec<u8>,
    hash:Hash
}

impl HashData {
    pub fn new(data:Vec<u8>,hash:Hash)->Self
    {
        HashData{data,hash}
    }
}

pub trait StorageStrategies where Self:AsRef<Path> + Send + Sync
{
    fn init_data_storage<'path>(self:Arc<Self>)->BoxFuture<'path,Result<File,io::Error>>;
    
    fn push_data<'path>(self:Arc<Self>,file:&'path mut File,data:Vec<u8>)->BoxFuture<'path,Result<(),io::Error>>
    {
        Box::pin(async move {
            file.write(&data).await?;
            Ok(())
        })
    }

    fn versionning<'path>(self:Arc<Self>,file:&'path mut File,data:HashData)->BoxFuture<'path,Result<(),io::Error>>;

}

//-------------------------------------------------------------------------

struct NormalFile { 
    parent: Arc<Path>,
}

impl<'path> From<&'path Path> for NormalFile {
    fn from(value: &'path Path) -> Self {
        NormalFile {parent: Arc::from(value)}
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

    fn versionning<'pinbox>(self:Arc<Self>,file:&'pinbox mut File,data:HashData)->BoxFuture<'pinbox,Result<(),io::Error>> {
        Box::pin(async move {
            tokio::fs::remove_file(self.as_ref()).await?;
            self.push_data(file, data.data).await?;
            Ok(())
        })
    }


}

//-------------------------------------------------------------------------

struct HashContainerFile {
    parent: Arc<Path>
}

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

    fn versionning<'pinbox>(self:Arc<Self>,file:&'pinbox mut File,data:HashData)->BoxFuture<'pinbox,Result<(),io::Error>> {
        Box::pin(async move {
            let mut pathbuf = self.as_ref().as_ref().to_path_buf();
            pathbuf.pop();
            pathbuf.push(Path::new(&data.hash.to_hex().to_string()));
            self.push_data(file, data.data).await?;
            Ok(())
        })
    }
}



pub enum StorageType {
    Normal,
    Hashed
}


impl From<usize> for StorageType {
    fn from(value: usize) -> Self {
        match value {
            x if x <= LARGE_FILE as usize => StorageType::Normal,
            _ => StorageType::Hashed
        }
    }
}

impl From<StorageType> for StorageStrategy {
    fn from(value: StorageType) -> Self {
        StorageStrategy { type_storage: value }
    }
}

pub struct StorageStrategy{
    type_storage:StorageType
}

impl StorageStrategy {

    pub async fn get_dyn_storage_strategy<'path>(&self,path: &'path Path)-> Box<Arc<dyn StorageStrategies + 'path>>
    {
        let binder:Arc<dyn StorageStrategies + 'path> = match self.type_storage {
            StorageType::Normal => Arc::from(NormalFile::from(path)),
            StorageType::Hashed => Arc::from(HashContainerFile::from(path))
        };
        Box::new(binder)
    }

}

//-------------------------------------------------------------------------