use tokio::sync::mpsc::Sender;
use crate::data::Observation;

pub type VendorDescriptor = String;

pub trait Vendor {
    type Config;
    type Error;

    fn new(name: String, config: Self::Config) -> Result<Self, Self::Error> where Self: Sized;

    async fn worker(self, tx: Sender<Observation>) -> Result<(), Self::Error>;
}

pub struct VendorInstance<V: Vendor> {
    pub(crate) descriptor: VendorDescriptor,
    pub(crate) vendor: V
}