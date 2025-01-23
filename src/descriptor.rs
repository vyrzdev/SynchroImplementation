use crate::square::{SquareListingDescriptor};

pub type GlobalListingDescriptor = Vec<String>; // Supports compound keys (variation-specific SKUs)

#[derive(Debug)]
pub enum ListingDescriptor {
    Square(SquareListingDescriptor)
}