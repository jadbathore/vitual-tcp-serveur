use std::{borrow::Cow, collections::HashMap, sync::Arc};

use fs_handler_wasi::structs::json_struct::JsonInfo;

use crate::{BUFFERS, JSON_PAYLOADS, PAYLOADS, enums::errors::GlobalError};

type TcpItem<'item> = (&'item[Arc<[u8]>],Arc<Cow<'item,str>>,&'item JsonInfo<'item>);

pub struct TcpSharedContentCollection
{
    data:&'static Vec<Arc<[u8]>>,
    payloads:&'static Vec<Arc<Cow<'static,str>>>,
    json_payloads:&'static Vec<JsonInfo<'static>>,
    index:Arc<HashMap< &'static str, usize>>,
    slice_data:Arc<Vec<(usize,usize)>>
}


impl TcpSharedContentCollection
{
    pub fn new()->Result<Self,GlobalError>
    {
        if let (Some(data),Some(payloads),Some(json_payloads)) = (BUFFERS.get(),PAYLOADS.get(),JSON_PAYLOADS.get())
        {
            let mut hash:HashMap<&'static str , usize> = HashMap::new();
            let mut data_slice:Vec<(usize, usize)> = Vec::new();
            let mut index_data:usize = 0;
            for (index,payload) in json_payloads.iter().enumerate() {
                // let a:Cow<'static, str> = payload.get_url();
                hash.insert(payload.get_url(), index);
                let slice_end:usize =  index_data + payload.get_chunks();
                data_slice.push((index_data,slice_end));
                index_data += payload.get_chunks();
            }
            // dbg!(hash);
            Ok( TcpSharedContentCollection { 
                    data:data,
                    payloads:payloads, 
                    json_payloads :json_payloads,
                    index: Arc::new(hash),
                    slice_data: Arc::new(data_slice)
                })
        } else {
            Err(GlobalError::UninitializedVariable)
        }
    }

    pub fn iter<'a>(&'a self) -> TcpSharedContentIterator<'a>
    {
        TcpSharedContentIterator::new(self)
    }

    pub fn search<'a>(&self,url:&str)-> Option<TcpItem<'a>>
    {
        if let Some(index ) = self.index.get(&url) {
            let (start,end) = self.slice_data[*index];
            return Some((&self.data[start..end],Arc::clone(&self.payloads[*index]),&self.json_payloads[*index]));
        }
        None
    }
}

pub struct TcpSharedContentIterator<'collection> {
    index_slice_data:usize,
    index:usize,
    shared_content_collection:& 'collection TcpSharedContentCollection
    
}

impl<'collection> TcpSharedContentIterator<'collection>
{
    fn new(payload_collection:& 'collection TcpSharedContentCollection)->Self
    {
        TcpSharedContentIterator { 
            index: 0,
            index_slice_data:0,
            shared_content_collection: payload_collection 
        }
    }

    fn is_valid(&self)->bool
    {
        self.index < self.shared_content_collection.payloads.len()
    }
}


impl<'collection> Iterator for  TcpSharedContentIterator<'collection> 
{
    type Item = TcpItem<'collection>;

    fn next(&mut self) -> Option<Self::Item>
    { 
        if self.is_valid() {
            let chunk_to_add = self.index_slice_data + &self.shared_content_collection.json_payloads[self.index].get_chunks();
            let data_current:&[Arc<[u8]>] = &self.shared_content_collection.data[self.index_slice_data..chunk_to_add];
            let json_current:&JsonInfo<'collection> = &self.shared_content_collection.json_payloads[self.index];
            let payload_request =  Some(
                (data_current,
                Arc::clone(&self.shared_content_collection.payloads[self.index]),
                json_current)
            );
            self.index_slice_data +=  &self.shared_content_collection.json_payloads[self.index].get_chunks();
            self.index += 1;
            return payload_request;
        }
        None
    }
}