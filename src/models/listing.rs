use std::collections::HashMap;
use uuid::Uuid;
use crate::data::{Field, Instance, Window};
use crate::poll::Poll;
use crate::vendors::square::SquareListingDescriptor;

pub type GlobalListingDescriptor = Vec<String>; // Supports compound keys (variation-specific SKUs)

#[derive(Debug)]
pub enum ListingDescriptor {
    Square(SquareListingDescriptor)
}

#[derive(Debug)]
pub struct Listing {
    pub(crate) descriptor: GlobalListingDescriptor,
    pub(crate) instances: HashMap<Uuid, ListingInstance>
}

#[derive(Debug)]
pub struct ListingInstance {
    pub(crate) last_updated: Window,
    pub(crate) history: Vec<ListingState>,
}

#[derive(Debug)]
pub struct ListingState {
    pub(crate) at: Window,
    pub(crate) descriptor: ListingDescriptor,
    pub(crate) fields: ListingFields,
}

#[derive(Debug)]
pub struct ListingFields {
    pub(crate) title: Option<ListingTitleField>,
}

impl Instance for ListingInstance {
    type State = ListingState;
    type GlobalDescriptor = GlobalListingDescriptor;
}


// Fields
#[derive(Debug, PartialEq)]
pub struct ListingTitleField {
    pub(crate) title: String,
    // pub(crate) history: Vec<Poll<String>>
}
impl Field<String> for ListingTitleField {}