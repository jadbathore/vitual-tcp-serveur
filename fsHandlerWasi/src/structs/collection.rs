use std::{num::TryFromIntError};

use crate::structs::{ payload_request::DataFile};

// #[derive(Clone)]
pub struct PayloadCollection<'paths,'collection>
{
    payloads:&'collection mut [DataFile<'paths>],
}

impl<'paths,'collection> PayloadCollection<'paths,'collection>
{

    pub fn from(buffer:&'paths mut Vec<DataFile<'paths>>)->Self
    {
        PayloadCollection{ payloads:buffer }
    }

    pub fn iter(&'collection self) -> PayloadIterator<'paths,'collection>
    {
        PayloadIterator::new(self)
    }
}

pub struct PayloadIterator<'paths,'collection> {
    index:usize,
    capacity_allocate: u64,
    payload_collection:& 'collection PayloadCollection<'paths,'collection>
    
}

impl<'paths,'collection> PayloadIterator<'paths,'collection>
{
    fn new(payload_collection:&'collection PayloadCollection<'paths,'collection>)->Self
    {
        PayloadIterator { index: 0,capacity_allocate:0 ,payload_collection: payload_collection }
    }

    pub fn add_capacity_to_allocate(&mut self,capacity:u64)
    {
        self.capacity_allocate += capacity;
    }

    fn is_valid(&self)->bool
    {
        self.index < self.payload_collection.payloads.len()
    }

    pub fn try_get_capacity_allocate(&mut self)-> Result<usize,TryFromIntError>
    {
        self.capacity_allocate.try_into()
    }

    pub fn reverse(&mut self){
        self.index = 0;
    } 
}


impl<'paths,'collection> Iterator for  PayloadIterator<'paths,'collection> 
{
    type Item = &'collection DataFile<'paths>;

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
