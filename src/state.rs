use crate::descriptor::{GlobalListingDescriptor, ListingDescriptor};
use crate::Instance;

struct Listing {
    descriptor: GlobalListingDescriptor,
    instances: Vec<ListingInstance>
}

#[derive(Debug)]
pub struct ListingInstance {
    pub(crate) descriptor: ListingDescriptor,
    pub(crate) fields: Option<ListingFields> // May not be populated.
}

impl Instance for ListingInstance {
    type Fields = ListingFields;
    type GlobalDescriptor = GlobalListingDescriptor;
}

#[derive(Debug)]
pub struct ListingFields {
    pub(crate) title: String,
}