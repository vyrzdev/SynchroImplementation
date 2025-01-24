use crate::data::Instance;

pub trait Vendor<I: Instance> {
    type Descriptor;
    type Error;

    // fn vend(&self, descriptor: &Self::Descriptor) -> Vec<I>; // Will given a descriptor suited to itself, vend a fields.

    async fn index(&self, cursor: Option<String>) -> Result<Vec<(I::GlobalDescriptor, I)>, Self::Error>;
}