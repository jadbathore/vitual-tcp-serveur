use std::sync::Arc;
use futures::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::{general::WriteSender, runtime::FakeToSubPath, structs::{async_strategies::FileAsyncReader, payloads::payload::DataFile}};



pub trait SearchableItem {}

pub type TcpItem = (&'static[Arc<[u8]>],Arc<String>);

impl<'item> SearchableItem for TcpItem {}
impl<'item> SearchableItem for &'item DataFile<FileAsyncReader<FakeToSubPath>> {}

#[derive(Default,Debug)]
pub struct IndexSliceHelper {
    len_slice:usize
}

impl IndexSliceHelper
{
    pub fn append_slice(&mut self,slice_gap:usize,slice:&mut Vec<(usize,usize)>)
    {
        let slice_end:usize =  self.len_slice + slice_gap;
        slice.push((self.len_slice,slice_end));
        self.len_slice = slice_end;
    }
}



pub trait PayloadSender:Iterator<Item:SearchableItem>
{
    type Collection: StaticCollection<StaticElement = Self::Item> + 'static;

    fn get_collection(&self)-> &'static Self::Collection;
    

    async fn send_payload(&self,write:&mut WriteSender,item:Self::Item);

    fn get_item(&self,index:usize)->Option<Self::Item>;

    async fn write_splitsink(&self,write:&mut WriteSender)
    {
        for i in self.get_collection().iter() {
            self.send_payload(write, i).await;
        }
    }
}

pub trait  PayloadCloser {
    async fn end_com(&self,write:&mut WriteSender){
        let _ = write.send(Message::Text(String::from("end"))).await;
        let _ = write.send(Message::Close(None)).await;
    }
}

pub trait StaticCollection {
    type StaticElement:SearchableItem + 'static;
    type Iter:Iterator<Item = Self::StaticElement> + 'static;
    fn iter(&'static self)-> Box<Self::Iter>;
    fn length(&self)->usize;
}