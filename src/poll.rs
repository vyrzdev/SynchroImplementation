use std::collections::HashMap;
use std::sync::Arc;
use hifitime::prelude::*;
use squareup::models::errors::SquareApiError;
use tokio::sync::mpsc::Sender;
use tokio::task::yield_now;
use uuid::Uuid;
use crate::data::Instance;
use crate::models::listing::{GlobalListingDescriptor, Listing};
use crate::models::listing::ListingDescriptor::Square;
use crate::vendor::Vendor;
use crate::vendors::square::SquareVendor;

#[derive(Debug)]
pub struct Poll<Type> {
    pub(crate) value: Type,
    pub(crate) sent: Epoch,
    pub(crate) received: Epoch
}


pub async fn vendor_worker<I: Instance, V: Vendor<I>>(vendor_id: Uuid, vendor: V, output: Sender<(I::GlobalDescriptor, Uuid, I::State)>) -> Result<(), V::Error> {
    loop {
        for (global_descriptor, state) in vendor.index(None).await? {
            output.send((global_descriptor, vendor_id, state)).await.expect("Couldn't send states!");
        }
        yield_now().await;
    }
    Ok(())
}


// pub async fn worker(vendor1_id: Uuid, square: SquareVendor, mut index: HashMap<GlobalListingDescriptor, Listing>) -> Result<(), SquareApiError> {
//     let TEST_PRODUCT: GlobalListingDescriptor = vec!["Foo4".to_string()];
//     println!("WORKING");
//     let listing = index.get_mut(&TEST_PRODUCT).unwrap();
//     loop {
//         for (vendor_id, i) in &mut listing.instances {
//             if (vendor_id == &vendor1_id) {
//                 let Square(desc) = &i.history.last().expect("Should have at least one record.").descriptor;
//                 let state = square.vend(desc).await?.expect("Listing doesn't exist!");
//                 i.history.push(state);
//                 // println!("{:#?}", listing);
//             }
//         }
//         println!("WORKING");
//         println!("{:#?}", listing);
//     }
//     Ok(())
// }


