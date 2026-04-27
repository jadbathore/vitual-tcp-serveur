use std::{ collections::HashMap, sync::{Arc, OnceLock}};
use colored::Colorize;
use commun_utils_handler::errors::GlobalError;
use futures::SinkExt;
use tokio_tungstenite::tungstenite::Message;
// use std::{ num::TryFromIntError,sync::OnceLock };

use crate::{ ASSETS, CACHES, PAYLOADS, general::Asset, structs::{
        builder::wasi::build_wasi_call, iterator::utils::{ IndexSliceHelper, PayloadCloser, PayloadSender, SearchableItem, StaticCollection, TcpItem, WriteSender}, payloads::{json_struct::JsonInfo, payload::DataFile}
    }};


//---------------------------------------------------------------------------
//-------------------------   utils function   ------------------------------
//---------------------------------------------------------------------------

fn adding_accesor_asset<SI,I,S,C>(static_collection:&'static OnceLock<Arc<S>>,insert_callback:&mut C)
    where 
        SI:SearchableItem,
        I:PayloadSender<Item = SI>,
        S:StaticCollection<Iter = I> + 'static,
        C: FnMut(usize,&SI)
{
    if let Some(collection) = static_collection.get() {
        for (index,item) in collection.iter().enumerate() {
            insert_callback(index,&item);
        }
    }
}

async fn handle_index_search<SI,I,S>(static_collection:&'static OnceLock<Arc<S>>,writer:&mut WriteSender,index:&usize)
    where   
        SI:SearchableItem,
        I:PayloadSender<Item = SI> + PayloadCloser,
        S:StaticCollection<Iter = I> + 'static
{
    if let Some(collection) = static_collection.get() {
        let iter = collection.iter();
        let item = iter.get_item(*index);
        if let Some(item)  = item {
            iter.send_payload(writer, item).await;
        }
        iter.end_com(writer).await;
    }
}

async fn handle_writer<SI,I,S>(static_collection:&'static OnceLock<Arc<S>>,writer:&mut WriteSender)
    where   
        SI:SearchableItem,
        I:PayloadSender<Item = SI>,
        S:StaticCollection<Iter = I> + 'static
{
    if let Some(collection) = static_collection.get() {
        collection.iter().write_splitsink(writer).await;
    }
}

//---------------------------------------------------------------------------
//--------------------- static collection assets ----------------------------
//---------------------------------------------------------------------------

pub struct PayloadCollection
{
    pub payloads:Vec<DataFile>,
}

impl TryFrom<Vec<DataFile>> for PayloadCollection
{
    type Error = Box<GlobalError>;
    fn try_from(data_files: Vec<DataFile>) -> Result<Self,Self::Error> {
        if let Some(_) = PAYLOADS.get() {
            return Err(Box::new(GlobalError::SingleInstanceBreach));
        }
        Ok(PayloadCollection { 
            payloads: data_files,
        })
    }
}

impl StaticCollection for PayloadCollection {
    type StaticElement = &'static DataFile;

    type Iter = StaticAssetIterator<PayloadCollection>;

    fn iter(&'static self)-> Box<Self::Iter> {
        Box::new(StaticAssetIterator::new(self))
    }

    fn length(&self)->usize {
        self.payloads.len()
    }
}

#[derive(Default,Debug)]
pub struct CacheCollection
{
    pub(crate) data:Vec<Arc<[u8]>>,
    pub(crate) payloads_stringify:Vec<Arc<String>>,
    pub(crate) slice_index: Vec<(usize,usize)>,
    // json_infos:Vec<JsonInfo>
    // pub accesor:IndexSliceAccessor<String>
}

impl TryFrom<Vec<DataFile>> for CacheCollection {
    type Error = Box<GlobalError>;
    fn try_from(data_files: Vec<DataFile>) -> Result<Self, Self::Error> {
        if let Some(_) = CACHES.get() {
            return Err(Box::new(GlobalError::SingleInstanceBreach));
        }
        let mut json_infos_stringity = Vec::new();
        let mut datas:Vec<Arc<[u8]>> = Vec::new();
        let mut slice:Vec<(usize,usize)> = Vec::new();
        let mut helper:IndexSliceHelper = IndexSliceHelper::default();
        for data_file in data_files.iter()
        {
            helper.append_slice(data_file.get_payload().get_chunks(),&mut slice);
            data_file.flush_data(&mut datas).map_err(|_|GlobalError::JsonSerialize)?;
            json_infos_stringity.push(Arc::new(data_file.get_payload().stringify_to_json()?));
        }
        Ok(CacheCollection {
            payloads_stringify:json_infos_stringity,
            data:datas,
            slice_index:slice
        })
    }
}



impl StaticCollection for CacheCollection {
    type StaticElement = TcpItem;
    type Iter = StaticAssetIterator<Self>;

    fn iter(&'static self)-> Box<Self::Iter> {
        Box::new(StaticAssetIterator::new(self))
    }

    fn length(&self)->usize {
        self.payloads_stringify.len()
    }
}

//---------------------------------------------------------------------------
//--------------------- accessor Assets element  ----------------------------
//---------------------------------------------------------------------------

pub struct StaticAssetsCollection {
    accesor:HashMap<String,Asset>
}

impl StaticAssetsCollection {

    pub fn new()->Result<Self,Box<GlobalError>>
    {
        if let Some(_) = ASSETS.get() {
            return Err(Box::new(GlobalError::SingleInstanceBreach));
        }
        let mut hash_asset = HashMap::new();
        adding_accesor_asset(&CACHES, &mut |index,(_,payloads)|{
            let json_info:JsonInfo = serde_json::from_str(payloads).unwrap();
            hash_asset.insert(json_info.get_url(), Asset::Cache(index));
        });
        adding_accesor_asset(&PAYLOADS, &mut |index,item|{
            hash_asset.insert(item.get_string_lossy_url().to_string(), Asset::Payload(index));
        });
        Ok(StaticAssetsCollection { accesor: hash_asset })
    }

    pub async fn search(&self,query:String,writer:&mut WriteSender) 
    {
        
        if let Some(item) = self.accesor.get(&query) {
            match item {
                Asset::Cache(i) => {
                    handle_index_search(&CACHES, writer, i).await;
                },
                Asset::Payload(i)=> {
                    handle_index_search(&PAYLOADS, writer, i).await;
                }
            }
        } else {
            println!("{}:the query '{}' not in the system","warning".yellow(),query.as_str());
        }
    }

    pub async fn write_all(&self,writer:&mut WriteSender)
    {
        handle_writer(&PAYLOADS, writer).await;
        handle_writer(&CACHES, writer).await;
        self.end_com(writer).await;
    }

    pub async fn exec(&self,command:String,writer:&mut WriteSender)
    {
        build_wasi_call::<(String,),()>((command,), "exec-utils").unwrap();
    }
}


pub struct StaticAssetIterator<T:'static +?Sized> {
    index:usize,
    payload_collection:&'static T
}

impl<T:StaticCollection + 'static> StaticAssetIterator<T>
{
    pub fn new(payload_collection:&'static T)->Self
    {
        StaticAssetIterator { 
            index: 0,
            // capacity_allocate:0 ,
            payload_collection: payload_collection  
        }
    }

    pub fn is_valid(&self)->bool
    {
        self.index < self.payload_collection.length()
    }

    pub fn index_exist(&self,index:usize)->bool 
    {
        index < self.payload_collection.length()
    }
}

//-----------------------------------------------------------------
//---------------------   impl iterator  --------------------------
//-----------------------------------------------------------------

impl Iterator for  StaticAssetIterator<PayloadCollection>
{
    type Item = &'static DataFile;

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.is_valid() {
            let payload_request =  Some(&self.payload_collection.payloads[self.index]);
            self.index += 1;
            return payload_request;
        }
        None
    }
}

impl Iterator for StaticAssetIterator<CacheCollection>
{
    type Item = TcpItem;

    fn next(&mut self) -> Option<Self::Item>
    { 
        if self.is_valid() {
            let (start,end) = self.payload_collection.slice_index[self.index];
            let data_current:&[Arc<[u8]>] = &self.payload_collection.data[start..end];
            let payload_request =  Some((data_current,Arc::clone(&self.payload_collection.payloads_stringify[self.index])));
            self.index += 1;
            return payload_request;
        }
        None
    }
}


//-----------------------------------------------------------------
//------------------ impl Payload traits --------------------------
//-----------------------------------------------------------------


impl PayloadSender for StaticAssetIterator<PayloadCollection>
{
    type Collection = PayloadCollection;
    
    fn get_collection(&self)-> &'static Self::Collection 
    {
        self.payload_collection
    }

    fn get_item(&self,index:usize)->Option<Self::Item> {
        if self.index_exist(index) {
            return Some(&self.payload_collection.payloads[index]);
        }
        None
    }

    async fn send_payload(&self,write:&mut WriteSender,item:&'static DataFile) {
        let mut datas:Vec<Arc<[u8]>> = Vec::new();

        if let (Ok(_),Ok(payload)) = (item.flush_data(&mut datas),item.get_payload().stringify_to_json())
        {
            let _ = write.send(Message::Text(payload)).await;
            for data in datas {
                let _ = write.send(Message::Binary(data.to_vec())).await;
            }
        } else {
            println!("can't read data or send payload for:'{}'",item.get_string_lossy_url())
        }
    }
}

impl PayloadCloser for StaticAssetIterator<PayloadCollection>{}

impl PayloadSender for StaticAssetIterator<CacheCollection>
{
    type Collection = CacheCollection;
    
    fn get_collection<'iter>(&'iter self)->&'static Self::Collection {
        self.payload_collection
    }

    fn get_item(&self,index:usize)->Option<Self::Item> 
    {
        if self.index_exist(index) {
            let (start,end) = self.payload_collection.slice_index[index];
            return Some((&self.payload_collection.data[start..end],Arc::clone(&self.payload_collection.payloads_stringify[index])));
        } 
        None
    }

    async fn send_payload(&self,write:&mut WriteSender,item:TcpItem) 
    {
        let (datas,payload) = item;
        let _ = write.send(Message::Text(payload.to_string())).await;
        for data in datas {
            let _ = write.send(Message::Binary(data.to_vec())).await;
        }
    }
}

impl PayloadCloser for StaticAssetIterator<CacheCollection>{}

impl PayloadCloser for StaticAssetsCollection{}

// impl Iterator for StaticAssetIterator<PayloadCollection>
// {
//     type Item = &'static DataFile;

//     fn next(&mut self) -> Option<Self::Item>
//     {
//         if self.is_valid() {
//             let payload_request =  Some(&self.payload_collection.payloads[self.index]);
//             self.index += 1;
//             return payload_request;
//         }
//         None
//     }
// }

// struct StaticAssetIterator<T:'static>
// {
//     index:usize,
//     payload_collection:&'static T
// }

// impl PayloadSender for StaticAssetIterator<CacheCollection>
// {
//     type Collection = CacheCollection;
    
//     fn collection<'iter>(&'iter self)-> &'static Self::Collection {
//         self.payload_collection
//     }

//     fn get_item(&self,index:usize)-> Option<Self::Item>
//     {
//         // self.is_valid();
//         // if let Some(index) = self.collection.accesor.get_index(url) {
//         //     let (start,end) = self.collection.accesor.get_slice(*index);
//         //     return Some((&self.collection.data[start..end],Arc::clone(&self.collection.payloads_stringify[*index])));
//         // }
//         None
//     }

//     async fn send_payload(&self,write:&mut WriteSender,item:TcpItem) 
//     {
//         let (datas,payload) = item;
//         let _ = write.send(Message::Text(payload.to_string())).await;
        
//         for data in datas {
//             let _ = write.send(Message::Binary(data.to_vec())).await;
//         }
//     }
// }

// impl Iterator for StaticAssetIterator<CacheCollection>
// {
//     type Item = TcpItem;
//     fn next(&mut self) -> Option<Self::Item>
//     { 
//         if self.is_valid() {
//             let (start,end) = self.payload_collection.accesor.get_slice(self.index);
//             let data_current:&[Arc<[u8]>] = &self.payload_collection.data[start..end];
//             let payload_request =  Some((data_current,Arc::clone(&self.payload_collection.payloads_stringify[self.index])));
//             self.index += 1;
//             return payload_request;
//         }
//         None
//     }
// }

// impl<C,P> PartialEq for AssetSender<C,P>
// where 
//         C:StaticCollection<Iter=StaticAssetIterator<CacheCollection>>,
//         P:StaticCollection<Iter=StaticAssetIterator<PayloadCollection>> 
// {
//     fn eq(&self, other: &Self) -> bool {

//         if let 
//         match other {
//             AssetSender::Cache(_) => {
                
//             }
//         }
//     }
// }

