mod ts_rs_decimal_fields;
mod ts_rs_export_sea_orm_entity_name;
mod stringify_ulid_identifiers;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn ts_rs_export_sea_orm_entity_name(args: TokenStream, input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);

    match ts_rs_export_sea_orm_entity_name::expand(derive_input, args) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn ts_rs_decimal_fields(_args: TokenStream, input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);

    match ts_rs_decimal_fields::expand(derive_input) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}


#[proc_macro_attribute]
pub fn stringify_ulid_identifiers(_args: TokenStream, input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);

    match stringify_ulid_identifiers::expand(derive_input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
