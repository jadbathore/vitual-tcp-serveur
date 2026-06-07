use std::{
    path::{Path,PathBuf}, 
    sync::Arc
};
use blake3::Hash;
use futures::io;
use tokio::{fs::{self,File}, io::AsyncWriteExt}; 
use std::ops::Deref;
use commun_utils_handler::fs_strategies::LARGE_FILE;

use crate::{general::{BoxFuture, ReadSender}, runtime::FakeToSubPath, structs::async_strategies::FileAsyncReader};



trait VersionningData {}


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


impl VersionningData for Vec<u8> {}
impl VersionningData for Hash {}

pub trait StorageStrategies where Self:AsRef<Path>
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
            self.push_data(file, data.data);
            Ok(())
        })
    }
}


pub struct StorageStrategy<'path>{
    strat:Arc<dyn StorageStrategies + 'path> 
}


impl<'path> StorageStrategy<'path>{
    pub fn new(path:&'path Path,predicate:usize)->Self
    {
        let arc:Arc<dyn StorageStrategies + 'path>  = match predicate {
            x if x <= LARGE_FILE as usize => Arc::from(NormalFile::from(path)),
            _ => Arc::from(HashContainerFile::from(path))
        };
        StorageStrategy { strat: arc }

    }
    
    pub async fn storage_strategy(&'path self)->Result<File, std::io::Error>
    {
        self.strat.clone().init_data_storage().await
    }

    pub async fn versionning(&'path self,file:&'path mut File, data:HashData)->Result<(), std::io::Error>
    {
        self.strat.clone().versionning(file, data).await;
        Ok(())
    }

}






// pub async fn storage_strategy<'path>(path:&'path Path,predicate:usize)->Result<File, std::io::Error>
// {
//     let storage_type:Arc<dyn StorageStrategies> = get_dyn_storage_strategy(path, predicate);
//     storage_type.init_data_storage().await 
// }





//-------------------------------------------------------------------------