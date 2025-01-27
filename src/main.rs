mod data;
mod vendor;
mod models;
mod vendors;
mod config;

extern crate hifitime;
extern crate serde_json;
extern crate squareup;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use futures::task::SpawnExt;
use serde::Serialize;
use tokio;
use tokio::sync::mpsc;
use tokio::task::{JoinSet};
use crate::config::{Config, VendorConfig};
use crate::data::{EntityDescriptor, Observation, Window, OVERLAP};
use crate::models::listing::{ListingDescriptor, ListingField};
use crate::vendor::{Vendor, VendorInstance};
use crate::vendors::square::{SquareVendor};

#[derive(Debug)]
struct Overlap {
    at: Window,
    observations: Vec<Observation>
}

#[derive(Debug)]
struct FieldState {
    history: Vec<Observation>,
    overlaps: Vec<Overlap>
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = serde_json::from_slice(&fs::read("config.json").expect("Failed to read config.json!")).expect("Failed to parse config!");

    let mut vendors = Vec::with_capacity(config.vendors.len());
    for (name, config) in config.vendors {
        vendors.push(match config {
            VendorConfig::Square(square_config) => VendorInstance {
                descriptor: name.clone(),
                vendor: SquareVendor::new(name, square_config)?,
            }
        });
    }

    let (tx,mut rx) = mpsc::channel::<Observation>(100);

    let mut workers = JoinSet::new();
    for mut vendor_instance in vendors {
        workers.spawn(vendor_instance.vendor.worker(tx.clone()));
    }

    futures::join!(workers.join_all(), coordinator(rx));
    Ok(())
}



async fn coordinator(mut rx: mpsc::Receiver<Observation>) {
    // Initialise state-tracking.

    let mut listing_state: HashMap<ListingDescriptor, HashMap<ListingField, FieldState>> = HashMap::new();

    while let Some(observation) = rx.recv().await {
        println!("Received Observation: {:#?}", observation);

        match &observation.subject {
            EntityDescriptor::ListingField((listing_descriptor, listing_field)) => {
                let listing = listing_state.entry(listing_descriptor.clone()).or_insert(HashMap::new());

                let field_state = listing.entry(listing_field.clone()).or_insert(FieldState {
                    overlaps: Vec::new(),
                    history: Vec::new()
                });

                field_state.history.push(observation);
                println!("New State added: {:#?}", field_state);
            }
        }
    }
}