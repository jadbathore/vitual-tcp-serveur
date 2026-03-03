
use std::{ 
    collections::HashMap, hash::Hash, ops::Deref, sync::Arc
};
use fs_handler_wasi::commun_utils::error::GlobalError;
use futures::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};


use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::structs::payloads::payload::DataFile;


pub trait SearchableItem {}

pub type TcpItem = (&'static[Arc<[u8]>],Arc<String>);

impl<'item> SearchableItem for TcpItem {}
impl<'item> SearchableItem for &'item DataFile {}

#[derive(Default)]
pub struct IndexAccessor<K: Eq + Hash> {
    index:HashMap<K,usize>,
    len_index:usize,
}

// impl<T: Copy + Eq + Hash + Default > From<Vec<T>> for IndexAccessor<T> {
//     fn from(value: Vec<T>) -> Self {
//         let mut default:IndexAccessor<T> = IndexAccessor::default();
//         for t in value.iter()
//         {   
//             default.append_index(*t);
//         }
//         default
//     }
// }


impl<K: Eq + Hash> IndexAccessor<K>
{
    pub fn append_index(&mut self,key:K)
    {
        self.index.insert(key,self.len_index);
        self.len_index += 1;
        // self.index.remove(k)
    }
    
    pub fn get_index(&self,key:K)->Option<&usize>
    {
        self.index.get(&key)
    }
}

impl<K: Eq + Hash> Deref for IndexSliceAccessor<K> {
    type Target = IndexAccessor<K>;
    
    fn deref(&self) -> &Self::Target {
        &self.accessor
    }
}

#[derive(Default)]
struct IndexSliceAccessor<K: Eq + Hash> {
    accessor:IndexAccessor<K>,
    slice:Vec<(usize,usize)>,
    len_slice:usize
}



impl<K: Eq + Hash> IndexSliceAccessor<K>
{
    fn append_slice(&mut self,key:K,slice_gap:usize)
    {
        self.accessor.append_index(key);
        let slice_end:usize =  self.len_slice + slice_gap;
        self.slice.push((self.len_slice,slice_end));
        self.len_slice = slice_end;
    }

    fn get_slice(&self,index:usize)->(usize,usize) 
    {
        self.slice[index]
    }
}

pub type WriteSender = SplitSink<WebSocketStream<TcpStream>,Message>;
pub type ReadSender = SplitStream<WebSocketStream<TcpStream>>;
pub trait PayloadSender:Iterator<Item:SearchableItem>
{
    type Collection: StaticCollection<Self::Item>;

    fn collection(&self)->&'static Self::Collection;

    fn search(&self,url:String)-> Option<Self::Item>;

    async fn send_payload(&self,write:&mut WriteSender,item:Self::Item);

    async fn end_com(&self,write:&mut WriteSender){
        let _ = write.send(Message::Text(String::from("end"))).await;
        let _ = write.send(Message::Close(None)).await;
    } 

    async fn write_splitsink(&self,write:&mut WriteSender)
        where <Self as PayloadSender>::Collection: 'static
    {
        // for a in self.cycle(){}
        for i in self.collection().iter() {
            self.send_payload(write, i).await;
        }
    }
    
    async fn read_splitsink(&self,write:&mut WriteSender,read:&mut ReadSender)
    {
        if let Some( message) = read.next().await {
            let data = message.unwrap_or(Message::binary(Vec::new())).into_text().unwrap_or(String::from(""));
            if let Some(tcp_item) = self.search(data) 
            {
                self.send_payload(write, tcp_item).await;
            }
        }
        self.end_com(write).await;
    }
}


pub trait StaticCollection<I> {
    type Iter:Iterator<Item=I> +'static;
    fn iter(&'static self)-> Box<Self::Iter>;
}

#[derive(Default)]
pub struct CacheCollection
{
    data:Vec<Arc<[u8]>>,
    payloads_stringify:Vec<Arc<String>>,
    // json_payloads:&'cache [JsonInfo<'cache>],
    accesor:IndexSliceAccessor<String>
}

impl StaticCollection<TcpItem> for CacheCollection {

    type Iter = CacheContentIterator;

    fn iter(&'static self)-> Box<Self::Iter> {
        Box::new(CacheContentIterator::new(self))
    }
}



// impl<'cache> PayloadSender<'cache> for CacheCollection {
//     type ItemSended = TcpItem<'cache>;
//     type Iter = CacheContentIterator<'cache>;

//     fn search(&'cache self,url:String)-> Option<Self::ItemSended> 
//     {
//         if let Some(index) = self.accesor.get_index(url) {
//             let (start,end) = self.accesor.get_slice(*index);
//             return Some((&self.data[start..end],Arc::clone(&self.payloads_stringify[*index])));
//         }
//         None
//     }

//     async fn send_payload(&'cache self,write:&mut WriteSender,item:Self::ItemSended) 
//     {
//         let (datas,payload) = item;
//         let _ = write.send(Message::Text(payload.to_string())).await;
//         for data in datas {
//             let _ = write.send(Message::Binary(data.to_vec())).await;
//         }
//     }

    

// }

impl TryFrom<Vec<DataFile>> for CacheCollection {

    type Error = Box<GlobalError>;

    fn try_from(data_files: Vec<DataFile>) -> Result<Self, Self::Error> {
        let mut payloads = Vec::new();
        let mut datas = Vec::new();
        let mut accessor = IndexSliceAccessor::default();

        for data_file in data_files.iter()
        {
            let url = data_file.get_string_lossy_url().to_string();
            accessor.append_slice(url, data_file.get_payload().get_chunks());
            data_file.flush_data(&mut datas).map_err(|_|GlobalError::JsonSerialize)?;
            payloads.push(Arc::new(data_file.get_payload().stringify_to_json()?));
        }

        Ok(CacheCollection {
            payloads_stringify:payloads,
            data:datas,
            accesor:accessor
        })
    }
}


impl PayloadSender for CacheContentIterator
{
    type Collection = CacheCollection;
    
    fn collection<'iter>(&'iter self)->&'static Self::Collection {
        self.collection
    }

    fn search(&self,url:String)-> Option<TcpItem> 
    {
        if let Some(index) = self.collection.accesor.get_index(url) {
            let (start,end) = self.collection.accesor.get_slice(*index);
            return Some((&self.collection.data[start..end],Arc::clone(&self.collection.payloads_stringify[*index])));
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




// impl CacheCollection
// {

//     // pub fn add(&mut self,data_file:DataFile)
//     // {   

//     //     self.accesor.append_slice(data_file.get_string_url().to_string(), header.get_chunks());
//     //     data_file.flush_data(&mut self.data);
//     // }

//     pub fn iter(&'static self) -> Box<CacheContentIterator>
//     {
//         Box::new(CacheContentIterator::new(self))
//     }
// }

pub struct CacheContentIterator {
    index:usize,
    collection:& 'static CacheCollection
}

impl CacheContentIterator
{
    fn new(payload_collection:& 'static CacheCollection)->Self
    {
        CacheContentIterator { 
            index: 0,
            collection: payload_collection 
        }
    }

    fn is_valid(&self)->bool
    {
        self.index < self.collection.payloads_stringify.len()
    }
}


impl Iterator for  CacheContentIterator
{
    type Item = TcpItem;

    fn next(&mut self) -> Option<Self::Item>
    { 
        if self.is_valid() {
            let (start,end) = self.collection.accesor.get_slice(self.index);
            let data_current:&[Arc<[u8]>] = &self.collection.data[start..end];
            let payload_request =  Some((data_current,Arc::clone(&self.collection.payloads_stringify[self.index])));
            self.index += 1;
            return payload_request;
        }
        None
    }
    
}