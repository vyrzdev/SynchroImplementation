use std::collections::HashMap;
use std::{vec, env};
use hifitime::Epoch;
use serde::{Deserialize, Serialize};
use squareup::api::CatalogApi;
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::errors::SquareApiError;
use squareup::models::{CatalogItem, ListCatalogParameters, ListCatalogResponse};
use squareup::SquareClient;
use tokio::sync::mpsc::Sender;
use crate::data::{Action, EntityDescriptor, Observation, Window};
use squareup::models::enums::CatalogObjectType;
use crate::models::listing::{ListingDescriptor, ListingField};
use crate::vendor::Vendor;

#[derive(Debug, Serialize, Deserialize)]
pub struct SquareConfig {
    pub(crate) token: String,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
enum SquareListingField {
    Title
}

impl From<SquareListingField> for ListingField {
    fn from(value: SquareListingField) -> Self {
        match value {
            SquareListingField::Title => ListingField::Title
        }
    }
}

struct SquareState {
    last_updated: Window,
    last_value: String,
}

pub(crate) struct SquareVendor {
    name: String,
    config: SquareConfig,
    catalog_api: CatalogApi,
    listing_state: HashMap<String, HashMap<SquareListingField, SquareState>>,
    listing_mapping: HashMap<String, ListingDescriptor>,
    // mapping: HashMap<String, EntityDescriptor>
}

struct SquareQuery<T> {
    at: Window,
    response: T
}

impl SquareVendor {
    async fn initialise(&mut self) -> Result<(), SquareApiError> {
        // List all listings...
        let mut params = ListCatalogParameters {
            catalog_version: None,
            types: Some(vec![CatalogObjectType::Item]),
            cursor: None
        };
        loop {
            let query = self.list_catalog_objects(&params).await?;
            params.cursor = query.response.cursor.clone();
            let mapping = Self::build_mapping_from_list_catalog_query(&query);
            let state = Self::process_list_catalog_query(&query);

            for (k, v) in mapping {
                self.listing_mapping.insert(k, v);
            }

            for (descriptor, fields) in state {
                let listing_fields = self.listing_state.entry(descriptor).or_insert(HashMap::new());
                for (field, value) in fields {
                    listing_fields.insert(field, SquareState {
                        last_updated: query.at.clone(),
                        last_value: value
                    });
                }
            }

            if params.cursor == None {
                break;
            }
        }

        Ok(())
    }

    // async fn process_

    async fn list_catalog_objects(&self, params: &ListCatalogParameters) -> Result<SquareQuery<ListCatalogResponse>, SquareApiError> {
        let sent = Epoch::now().expect("Failed to take timestamp!");
        let response = self.catalog_api.list_catalog(&params).await?;
        Ok(SquareQuery {
            at: (sent, Epoch::now().expect("Failed to take timestamp!") ),
            response
        })
    }

    fn find_listing_descriptor(listing: &CatalogItem) -> ListingDescriptor {
        let mut descriptor = Vec::new();

        let variations = listing.variations.as_ref().expect("Square requires 1 variation per listing.");
        for variation in variations {
            let variation_data = variation.item_variation_data.as_ref().expect("Variation had no data!");
            if let Some(sku) = variation_data.sku.as_ref() {
                descriptor.push(sku.clone());
            }
        }

        ListingDescriptor {
            sku: descriptor
        }
    }

    fn build_mapping_from_list_catalog_query(query: &SquareQuery<ListCatalogResponse>) -> Vec<(String, ListingDescriptor)> {
        let mut mapping = Vec::new(); // TODO: Mapping should happen here?

        match query.response.objects.as_ref() {
            Some(objects) => {
                for object in objects {
                    let listing_id = object.id.clone();
                    let listing_descriptor = Self::find_listing_descriptor(object.item_data.as_ref().expect("Item had no item data!"));

                    mapping.push((listing_id, listing_descriptor));
                }

                mapping
            },
            None => mapping
        }

    }

    fn process_list_catalog_query(query: &SquareQuery<ListCatalogResponse>) -> Vec<(String, Vec<(SquareListingField, String)>)> {
        let mut build = Vec::new();

        match query.response.objects.as_ref() {
            Some(objects) => {
                for object in objects {
                    build.push((
                        object.id.clone(),
                        Self::process_catalog_item(
                            query.at.clone(),
                            object.item_data.as_ref().expect("Item had no item data!")
                        )
                    ));
                }

                build
            },
            None => build
        }
    }

    fn process_catalog_item(at: Window, catalog_item: &CatalogItem) -> Vec<(SquareListingField, String)> {
        let mut fields = Vec::new();
        let title = catalog_item.name.as_ref().expect("Item had no name! (square requires this!)");
        fields.push((SquareListingField::Title, title.clone()));

        fields
    }
}

impl Vendor for SquareVendor {
    type Config = SquareConfig;
    type Error = SquareApiError;

    fn new(name: String, config: SquareConfig) -> Result<Self, Self::Error> {
        env::set_var("SQUARE_API_TOKEN", &config.token);
        let catalog_api = CatalogApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // OPTIONAL if you set the SQUARE_ENVIRONMENT env var
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        })?);

        Ok(Self { name, config, catalog_api, listing_state: HashMap::new(), listing_mapping: HashMap::new() })
    }

    async fn worker(mut self, tx: Sender<Observation>) -> Result<(), Self::Error> {
        self.initialise().await?; // Initialise mappings and state!
                                  // TODO: Load from file... Persistence

        loop {
            // List all listings...
            let mut params = ListCatalogParameters {
                catalog_version: None,
                types: Some(vec![CatalogObjectType::Item]),
                cursor: None
            };
            loop {
                let query = self.list_catalog_objects(&params).await?;
                params.cursor = query.response.cursor.clone();
                let state = Self::process_list_catalog_query(&query);

                for (descriptor, fields) in state {
                    let listing_fields = self.listing_state.entry(descriptor.clone()).or_insert(HashMap::new());
                    for (field, value) in fields {
                        let current_state = listing_fields.get_mut(&field).expect("Field should have been present during initialisation!");

                        if current_state.last_value != value {
                            println!("State Changed! ");
                            current_state.last_updated = query.at.clone();
                            current_state.last_value = value;

                            tx.send(Observation {
                                subject: EntityDescriptor::ListingField((
                                    self.listing_mapping.get(&descriptor).expect("Should be in mapping!").clone(),
                                    field.into()
                                )),
                                at: query.at.clone(),
                                source: self.name.clone(),
                                action: Action::Assignment,
                            }).await.expect("Failed to send through channel!");
                        } else {
                            current_state.last_updated = query.at.clone();
                        }
                    }
                }

                if params.cursor == None {
                    break;
                }
            }
        }
    }
}