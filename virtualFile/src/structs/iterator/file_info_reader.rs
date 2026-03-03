// use std::{ num::TryFromIntError,sync::OnceLock };
use futures::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::{
    structs::{
        iterator::cached_data::{ IndexAccessor, PayloadSender, StaticCollection, WriteSender}, 
        payloads::payload::DataFile
    }
};

pub struct PayloadCollection
{
    payloads:Vec<DataFile>,
    accesor:IndexAccessor<String>
}

impl From<Vec<DataFile>> for PayloadCollection
{
    fn from(data_files: Vec<DataFile>) -> Self {
        let mut accessor = IndexAccessor::default();
        for data_file in data_files.iter(){
            let url = data_file.get_string_lossy_url().to_string();
            accessor.append_index(url);
        }

        PayloadCollection { 
            payloads: data_files,
            accesor:accessor
        }
    }
}

impl StaticCollection<&'static DataFile> for PayloadCollection {
    type Iter = PayloadIterator;

    fn iter(&'static self)-> Box<Self::Iter> {
        Box::new(PayloadIterator::new(self))
    }
}

pub struct PayloadIterator {
    index:usize,
    payload_collection:&'static PayloadCollection
    
}

impl PayloadIterator
{
    fn new(payload_collection:&'static PayloadCollection)->Self
    {
        PayloadIterator { 
            index: 0,
            // capacity_allocate:0 ,
            payload_collection: payload_collection  
        }
    }

    // pub fn add_capacity_to_allocate(&mut self,capacity:u64)
    // {
    //     self.capacity_allocate += capacity;
    // }

    fn is_valid(&self)->bool
    {
        self.index < self.payload_collection.payloads.len()
    }
}

impl Iterator for  PayloadIterator
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

impl PayloadSender for PayloadIterator
{
    type Collection = PayloadCollection;
    
    fn collection(&self)-> &'static Self::Collection 
    {
        self.payload_collection
    }

    fn search(&self,url:String)-> Option<Self::Item>
    {
        if let Some(index) = self.payload_collection.accesor.get_index(url) {
            return Some(&self.payload_collection.payloads[*index]);
        }
        None
    }

    async fn send_payload(&self,write:&mut WriteSender,item:Self::Item) {
        let mut datas = Vec::new();
        if let (Ok(payload),Ok(_)) = (item.get_payload().stringify_to_json(),item.flush_data(&mut datas)) 
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


