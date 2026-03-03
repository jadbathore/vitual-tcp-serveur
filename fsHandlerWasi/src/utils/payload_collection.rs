pub struct DataCollection<T>
{
    payloads:Vec<T>,
}

impl<T> From<Vec<T>> for DataCollection<T>
{
    fn from(data_files: Vec<T>) -> Self {
        DataCollection { 
            payloads: data_files,
        }
    }
}

pub trait Collection<'collection,I> {
    type Iter:Iterator<Item=I> + 'collection;
    fn iter(&'collection self)-> Box<Self::Iter>;
}

impl<'collection,T> Collection<'collection,&'collection T> for DataCollection<T> {
    type Iter = DataIterator<'collection,T>;

    fn iter(&'collection self)-> Box<Self::Iter> {
        Box::new(DataIterator::new(self))
    }
}

pub struct DataIterator<'iter,T> {
    index:usize,
    payload_collection:&'iter DataCollection<T>
}

impl<'iter,T> DataIterator<'iter,T>
{
    fn new(payload_collection:&'iter DataCollection<T>)->Self
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
            let payload_request =  Some(&self.payload_collection.payloads[self.index]);
            self.index += 1;
            return payload_request;
        }
        None
    }
}