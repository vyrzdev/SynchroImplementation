use hifitime::Epoch;
use crate::models::listing::{ListingDescriptor, ListingField};
use crate::vendor::VendorDescriptor;

pub type Window = (Epoch, Epoch);

#[derive(Debug)]
pub enum EntityDescriptor {
    ListingField((ListingDescriptor, ListingField)),
}

#[derive(Debug)]
pub enum Action {
    Mutation,
    Assignment
}

#[derive(Debug)]
pub struct Observation {
    pub(crate) subject: EntityDescriptor,
    pub(crate) at: (Epoch, Epoch),
    pub(crate) source: VendorDescriptor,
    pub(crate) action: Action,
}


// Assume well-formed; a < b

//
// struct Observation<T> {
//     value: T,
//     window: Window,
// }

// // Checks if windows overlap;
// // See: https://stackoverflow.com/questions/3269434/whats-the-most-efficient-way-to-test-if-two-ranges-overlap
// const OVERLAP: fn(Window, Window) -> bool = |a, b| (a.0 <= b.1) && (b.0 <= a.1);