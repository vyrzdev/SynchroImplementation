use hifitime::Epoch;
use squareup::{
    api::CatalogApi,
    config::{BaseUri, Configuration, Environment},
    http::client::HttpClientConfiguration,
    SquareClient,
};
use squareup::models::enums::CatalogObjectType;
use squareup::models::errors::SquareApiError;
use squareup::models::{CatalogItem, CatalogObject, ListCatalogParameters, ListCatalogResponse, RetrieveCatalogObjectParameters};
use crate::Vendor;
use crate::models::listing::{GlobalListingDescriptor, ListingDescriptor, ListingFields, ListingInstance, ListingState, ListingTitleField};
use crate::poll::Poll;

#[derive(Debug)]
pub struct SquareListingDescriptor {
    catalog_object_id: String
}

pub struct SquareVendor {
    catalog_api: CatalogApi
}

pub struct Query<T> {
    sent: Epoch,
    received: Epoch,
    response: T
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


    async fn list_catalog_objects(&self, params: &ListCatalogParameters) -> Result<Query<ListCatalogResponse>, SquareApiError> {
        let sent = Epoch::now().expect("Failed to take timestamp!");
        let response = self.catalog_api.list_catalog(&params).await?;
        Ok(Query {
            received: Epoch::now().expect("Failed to take timestamp!"),
            sent,
            response
        })
    }

    fn find_listing_global_descriptor(listing: &CatalogItem) -> GlobalListingDescriptor {
        let mut descriptor = vec![];

        let variations = listing.variations.as_ref().expect("Square requires 1 variation per listing.");
        for variation in variations {
            let variation_data = variation.item_variation_data.as_ref().expect("Variation had no data!");
            if let Some(sku) = variation_data.sku.as_ref() {
                descriptor.push(sku.clone());
            }
        }

        descriptor
    }

    fn process_catalog_item(sent: Epoch, received: Epoch, catalog_object: CatalogObject) -> ListingState {
        let item_data = catalog_object.item_data.expect("Item had no item data!");
        let title = item_data.name.expect("Item had no name! (square requires this!)");

        ListingState {
            at: (sent, received),
            descriptor: ListingDescriptor::Square(SquareListingDescriptor {
                catalog_object_id: catalog_object.id
            }),
            fields: ListingFields {
                title: Some(ListingTitleField {
                    title: title
                })
            }
        }
    }

    fn process_list_catalog_query(query: Query<ListCatalogResponse>) -> Vec<(GlobalListingDescriptor, ListingState)> {
        let mut build = Vec::new();

        match query.response.objects {
            Some(objects) => {
                for object in objects {
                    build.push((
                        Self::find_listing_global_descriptor(object.item_data.as_ref().expect("Item had no item data!")),
                        Self::process_catalog_item(query.sent.clone(), query.received.clone(), object))
                    );
                }

                build
            },
            None => build
        }
    }
}

impl Vendor<ListingInstance> for SquareVendor {
    type Descriptor = SquareListingDescriptor;
    type Error = SquareApiError;

    async fn vend(&self, descriptor: &SquareListingDescriptor) -> Result<Option<ListingState>, SquareApiError> {
        let params = RetrieveCatalogObjectParameters {
            include_related_objects: None,
            catalog_version: None,
            include_category_path_to_root: None,
        };
        let sent = Epoch::now().expect("Failed to take timestamp!");
        let response = self.catalog_api.retrieve_catalog_object(
            &descriptor.catalog_object_id,
            &params
        ).await?;
        let received = Epoch::now().expect("Failed to take timestamp!");

        Ok(match response.object {
            Some(object) => Some(Self::process_catalog_item(sent, received, object)),
            None => None
        })
    }

    // fn vend(&self, descriptor: &Self::Descriptor) -> Result<Option<ListingInstance>, Self::Error> {
    //     let params = RetrieveCatalogObjectParameters {
    //
    //     };
    //
    //     let response = self.catalog_api.retrieve_catalog_object(
    //         &descriptor.catalog_object_id,
    //         &params).await?;
    //
    //     match &response.object {
    //         Some(listing) => self.consume(response.object),
    //         None => Ok(None)
    //     }
    //
    //
    // }

    async fn index(&self, cursor: Option<String>) -> Result<Vec<(GlobalListingDescriptor, ListingState)>, SquareApiError> {
        let mut index = Vec::new();

        let mut params = ListCatalogParameters {
            catalog_version: None,
            types: Some(vec![CatalogObjectType::Item]),
            cursor
        };
        loop {
            let query = self.list_catalog_objects(&params).await?;
            params.cursor = query.response.cursor.clone();

            index.append(&mut Self::process_list_catalog_query(query));

            if params.cursor == None {
                break;
            }
        }
        Ok(index)
    }
}