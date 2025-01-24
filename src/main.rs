mod data;
mod vendor;
mod models;
mod vendors;
extern crate squareup;
use std::collections::HashMap;
use std::env;
use tokio;
use crate::vendor::Vendor;
use crate::vendors::square::{SquareVendor};
use crate::models::listing::{Listing};


// Assume well-formed; a < b
// type Window = (SystemTime, SystemTime);
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

    let vendor1_index = vendor1.index(None).await.unwrap();
    let vendor2_index = vendor2.index(None).await.unwrap();

    let mut listing_record = HashMap::new();
    for (desc, instance) in vendor1_index {
        listing_record.insert(desc.clone(), Listing {
            descriptor: desc,
            instances: vec![instance],
        });
    }

    for (desc, instance) in vendor2_index {
        match listing_record.get_mut(&desc) {
            None => {
                listing_record.insert(desc.clone(), Listing {
                    descriptor: desc,
                    instances: vec![instance],
                });
            }
            Some(listing) => {
                listing.instances.push(instance);
            }
        }
    }

    println!("{:#?}", listing_record)
}