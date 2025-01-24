pub trait Instance {
    type GlobalDescriptor;
}

pub trait Field<Type: Clone> {
    fn clone_value(&self) -> Type;
}