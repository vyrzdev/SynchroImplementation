#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ListingDescriptor {
    pub(crate) sku: Vec<String>, // TODO: Partial Equality on Listing Descriptors.
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ListingField {
    Title
}