use crate::descriptor::{GlobalListingDescriptor, ListingDescriptor};
use crate::{Field, Instance};

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

// #[derive(Debug)]
// pub enum ListingFieldKind {
//     Title(ListingTitleField),
// }

#[derive(Debug)]
pub struct ListingTitleField {
    pub(crate) value: String,
}
impl Field<String> for ListingTitleField {
    fn clone_value(&self) -> String {
        self.value.clone()
    }
}


// #[derive(Debug)]
// pub struct ListingFields {
//     pub(crate) title: String,
// }