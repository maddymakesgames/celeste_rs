use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Error};

#[path = "enum.rs"]
mod yaml_enum;
#[path = "struct.rs"]
mod yaml_struct;

pub(super) fn yaml_derive(input: DeriveInput) -> Result<TokenStream, Error> {
    match input.data.clone() {
        Data::Struct(data_struct) => yaml_struct::yaml_derive_struct(input, data_struct),
        Data::Enum(data_enum) => yaml_enum::yaml_derive_enum(input, data_enum),
        Data::Union(_) => unreachable!("We check before running this function"),
    }
}
