use crate::data::{Field, Instance};
use crate::vendors::square::SquareListingDescriptor;

pub type GlobalListingDescriptor = Vec<String>; // Supports compound keys (variation-specific SKUs)

#[derive(Debug)]
pub enum ListingDescriptor {
    Square(SquareListingDescriptor)
}

#[derive(Debug)]
pub struct Listing {
    pub(crate) descriptor: GlobalListingDescriptor,
    pub(crate) instances: Vec<ListingInstance>
}

#[derive(Debug)]
pub struct ListingInstance {
    pub(crate) descriptor: ListingDescriptor,
    pub(crate) title: Option<ListingTitleField> // May not be populated.
}

impl Instance for ListingInstance {
    type GlobalDescriptor = GlobalListingDescriptor;
}


// Fields
#[derive(Debug)]
pub struct ListingTitleField {
    pub(crate) value: String,
}
impl Field<String> for ListingTitleField {
    fn clone_value(&self) -> String {
        self.value.clone()
    }
}