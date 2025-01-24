use crate::data::Instance;

pub trait Vendor<I: Instance> {
    type Descriptor;
    type Error;

    async fn vend(&self, descriptor: &Self::Descriptor) -> Result<Option<I::State>, Self::Error>; // Will given a descriptor suited to itself, vend a fields.

    async fn index(&self, cursor: Option<String>) -> Result<Vec<(I::GlobalDescriptor, I::State)>, Self::Error>;
}