#[derive(Debug, Clone)]
pub struct ListingDescriptor {
    pub(crate) sku: Vec<String>,
}

#[derive(Debug)]
pub enum ListingField {
    Title
}