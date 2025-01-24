use hifitime::Epoch;
use crate::models::listing::ListingState;

pub trait Instance {
    type GlobalDescriptor;
    type State;
}

pub trait Field<Type: Clone> {
    // fn clone_value(&self) -> Type;
}

pub type Window = (Epoch, Epoch);