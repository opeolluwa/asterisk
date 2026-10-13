use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, LitStr, parse_quote_spanned};

use crate::utils::convert_snake_to_pascal;

pub(crate) fn expand(
    mut input: DeriveInput,
    args: TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut literal_name = None;

    // Append attribute argument or default value of "Entity"
    let entity_name_append = syn::parse::<Option<LitStr>>(args)?
        .map(|x| x.value())
        .unwrap_or(String::from("Interface"));

    input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("sea_orm"))
        .try_for_each(|attr| {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table_name") || meta.path.is_ident("entity") {
                    literal_name = Some(meta.value()?.parse::<LitStr>()?);
                }
                Ok(())
            })
        })?;

    let literal_name = literal_name.ok_or(syn::Error::new(
        input.ident.span(),
        "Must provide a #[sea_orm(table_name = \"...\")] or #[sea_orm(entity = \"...\")] attribute",
    ))?;

    // Get the string value stored in the literal
    let ts_name = LitStr::new(
        &(convert_snake_to_pascal(&literal_name.value()) + &entity_name_append),
        literal_name.span(),
    );

    let mut attrs: Vec<Attribute> = vec![
        parse_quote_spanned!(input.ident.span() => #[derive(::ts_rs::TS)]),
        parse_quote_spanned!(literal_name.span() => #[ts(export, rename = #ts_name)]),
    ];

    let input_attrs: &mut Vec<Attribute> = input.attrs.as_mut();

    input_attrs.append(&mut attrs);

    Ok(quote!(#input))
}
