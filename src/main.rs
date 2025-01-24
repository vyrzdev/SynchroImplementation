mod data;
mod vendor;
mod models;
mod vendors;
mod poll;
extern crate hifitime;
extern crate squareup;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use squareup::models::errors::SquareApiError;
use tokio;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::task::{yield_now, JoinSet};
use uuid::Uuid;
use crate::vendor::Vendor;
use crate::vendors::square::{SquareVendor};
use crate::models::listing::{GlobalListingDescriptor, Listing, ListingInstance, ListingState};

// Assume well-formed; a < b

//
// struct Observation<T> {
//     value: T,
//     window: Window,
// }

// // Checks if windows overlap;
// // See: https://stackoverflow.com/questions/3269434/whats-the-most-efficient-way-to-test-if-two-ranges-overlap
// const OVERLAP: fn(Window, Window) -> bool = |a, b| (a.0 <= b.1) && (b.0 <= a.1);


#[tokio::main]
async fn main() {

    let vendor1 = SquareVendor::new();
    env::set_var("SQUARE_API_TOKEN", "");
    let vendor2 = SquareVendor::new();

    let vendors = HashMap::from([
        (Uuid::new_v4(), vendor1),
        (Uuid::new_v4(), vendor2)
    ]);

    let mut listings = HashMap::new();

    for (id, vendor) in &vendors {
        let vendor_index = vendor.index(None).await.unwrap();

        for (global_descriptor, listing_state) in vendor_index {
            match listings.get_mut(&global_descriptor) {
                None => {
                    listings.insert(global_descriptor.clone(), Listing {
                        descriptor: global_descriptor,
                        instances: HashMap::from([(id.clone(), ListingInstance {
                            last_updated: listing_state.at.clone(),
                            history: vec![listing_state],
                        })]),
                    });
                }
                Some(listing) => {
                    listing.instances.insert(id.clone(), ListingInstance {
                        last_updated: listing_state.at.clone(),
                        history: vec![listing_state],
                    });
                }
            }
        }
    }


    let (tx, mut rx) = mpsc::channel(100);

    let mut workset = JoinSet::default();
    for (id, vendor) in vendors {
        workset.spawn(poll::vendor_worker::<ListingInstance, SquareVendor>(id, vendor, tx.clone()));
    }

    futures::join!(workset.join_all(), coordinator(rx, listings));
}

async fn coordinator(mut rx: Receiver<(GlobalListingDescriptor, Uuid, ListingState)>, mut listings: HashMap<GlobalListingDescriptor, Listing>) {
    while let Some((global_descriptor, vendor_id, state)) = rx.recv().await {
        match listings.get_mut(&global_descriptor) {
            Some(listing) => match listing.instances.get_mut(&vendor_id) {
                Some(instance) => {
                    println!("Delved to instance: {:#?}", instance);
                    let last_state = instance.history.last().expect("Should be at least one state.");
                    let recorded_at= state.at.clone();
                    if last_state.fields.title != state.fields.title {
                        println!("State Change! Title Changed from {:#?} to {:#?}; Modification occurred from: {} to {}", last_state.fields.title, state.fields.title, instance.last_updated.0, state.at.1);
                        instance.history.push(state);
                    }
                    instance.last_updated = recorded_at;
                },
                None => {
                    println!("Unexpected! Found listing returned by vendor that is not yet registered for it.");
                }
            },
            None => {
                println!("Unexpected! Found listing returned by coordinators that is not registered in the listings.");
            }
        }
        yield_now().await;
    }
}