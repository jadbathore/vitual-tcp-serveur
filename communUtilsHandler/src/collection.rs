#[derive(Default)]
pub struct GenericCollection<'a,T>
{
    payloads:Vec<&'a T>,
}

impl<'a,T> From<Vec<&'a T>> for GenericCollection<'a,T>
{
    fn from(data_files: Vec<&'a T>) -> Self {
        GenericCollection { 
            payloads: data_files,
        }
    }
}


pub trait Collection<'collection,I> {
    type Iter:Iterator<Item=I> + 'collection;
    fn iter(&'collection self)-> Box<Self::Iter>;
    fn add(&'collection mut self,item:I);
    fn extend(&'collection mut self,items:&'collection[I]);
}

impl<'collection,T> Collection<'collection,&'collection T> for GenericCollection<'collection,T> {
    type Iter = DataIterator<'collection,T>;

    fn iter(&'collection self)-> Box<Self::Iter> {
        Box::new(DataIterator::new(self))
    }

    fn add(&'collection mut self,item:&'collection T) {
        self.payloads.push(item);
    }

    fn extend(&'collection mut self,items:&'collection[&'collection T]) {
        self.payloads.extend_from_slice(items);
    }

}

pub struct DataIterator<'iter,T> {
    index:usize,
    payload_collection:&'iter GenericCollection<'iter,T>
}

impl<'iter,T> DataIterator<'iter,T>
{
    fn new(payload_collection:&'iter GenericCollection<'iter,T>)->Self
    {
        DataIterator { index: 0, payload_collection } 
    }

    fn is_valid(&self)->bool
    {
        self.index < self.payload_collection.payloads.len()
    }
}

impl<'iter,T> Iterator for DataIterator<'iter,T>
{
    type Item = &'iter T;

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.is_valid() {
            let payload_request =  Some(self.payload_collection.payloads[self.index]);
            self.index += 1;
            return payload_request;
        }
        None
    }
}