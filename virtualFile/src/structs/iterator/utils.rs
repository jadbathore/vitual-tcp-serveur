use std::sync::Arc;
use futures::{SinkExt, stream::{SplitSink, SplitStream}};
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::structs::payloads::payload::DataFile;


pub trait SearchableItem {}

pub type TcpItem = (&'static[Arc<[u8]>],Arc<String>);

impl<'item> SearchableItem for TcpItem {}
impl<'item> SearchableItem for &'item DataFile {}

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

pub type WriteSender = SplitSink<WebSocketStream<TcpStream>,Message>;
pub type ReadSender = SplitStream<WebSocketStream<TcpStream>>;


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

// pub trait SingleToneInstanceCollection where Self: Default + 'static
// {
//     type Initializer;
//     const INSTANCE:&'static OnceLock<Arc<Self>>;

//     fn new(&self,instance:Self::Initializer)->Result<PhantomData<Self::Initializer>,GlobalError>;

//     fn init_from(&self,instance:Self::Initializer)->Result<&Self,GlobalError>
//     {
//         if let Some(_) = Self::INSTANCE.get() {
//             return Err(GlobalError::SingleInstanceBreach);
//         }
//         let a = self.new(instance)?;
//         Ok(&self)
//     }
// }

// impl SingleToneInstanceCollection for PayloadCollection {
//     type Initializer = Vec<DataFile>;
//     const INSTANCE:&'static OnceLock<Arc<Self>> = &PAYLOADS;

//     fn new(&self,initializer:Self::Initializer)->Result<PhantomData<Self::Initializer>,GlobalError> {
//         let a = Self {
//             payloads: initializer,
//             ressource_type:PhantomData
//         };
//         Ok(a.ressource_type)
//     }
// }