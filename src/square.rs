use std::env;
use crate::{Vendor, state::{ListingInstance}};

extern crate squareup;
use squareup::{
    api::CatalogApi,
    config::{BaseUri, Configuration, Environment},
    http::client::HttpClientConfiguration,
    SquareClient,
};
use squareup::models::enums::CatalogObjectType;
use squareup::models::errors::SquareApiError;
use squareup::models::{CatalogObject, ListCatalogParameters};
use squareup::models::enums::TenderType::SquareAccount;
use crate::descriptor::{GlobalListingDescriptor, ListingDescriptor};
use crate::state::ListingFields;

#[derive(Debug)]
pub struct SquareListingDescriptor {
    catalog_object_id: String
}

pub struct SquareVendor {
    catalog_api: CatalogApi
}

impl SquareVendor {
    pub(crate) fn new() -> Self {
        let config = Configuration {
            environment: Environment::Sandbox, // OPTIONAL if you set the SQUARE_ENVIRONMENT env var
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        };
        let catalog_api = CatalogApi::new(SquareClient::try_new(config).unwrap());
        SquareVendor { catalog_api }
    }

    fn process_listing(listing: CatalogObject) -> Option<(GlobalListingDescriptor, ListingInstance)> {
        // TODO: Builder Syntax <3
        let catalog_id = listing.id;
        let listing = listing.item_data?;
        let variation_data = listing.variations.expect("Square requires 1 variation min on a listing.");
        assert!(variation_data.len() >= 1); // Square requires one variation minimum per listing.

        let mut global_descriptor: GlobalListingDescriptor = Vec::with_capacity(variation_data.len());
        for variation in variation_data {
            let variation = variation.item_variation_data?;
            if let Some(sku) = variation.sku {
                global_descriptor.push(sku);
            }
        }
        if global_descriptor.len() == 0 {
            return None; // Product is UNTRACKED (NO SKU)
        }

        Some((global_descriptor, ListingInstance {
            descriptor: ListingDescriptor::Square(SquareListingDescriptor {
                catalog_object_id: catalog_id
            }),
            fields: Some(ListingFields {
                title: listing.name.expect("Square requires listings to have a name.")
            })
        }))
    }
}

impl Vendor<ListingInstance> for SquareVendor {
    type Descriptor = SquareListingDescriptor;
    type Error = SquareApiError;

    fn vend(&self, descriptor: &Self::Descriptor) -> ListingFields {
        todo!()
    }

    async fn index(&self, cursor: Option<String>) -> Result<Vec<(GlobalListingDescriptor, ListingInstance)>, SquareApiError> {
        let mut index = Vec::new();

        let mut params = ListCatalogParameters {
            catalog_version: None,
            types: Some(vec![CatalogObjectType::Item]),
            cursor
        };
        loop {
            let response = self.catalog_api.list_catalog(&params).await?;
            params.cursor = response.cursor;

            if let Some(objects) = response.objects {
                // Process...

                for object in objects {
                    match SquareVendor::process_listing(object) {
                        Some(record) => index.push(record),
                        None => {
                            println!("Ignored listing as process returned None!");
                        }
                    }
                }
            }

            if params.cursor == None {
                break;
            }
        }
        Ok(index)
    }
}