use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub(crate) fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let mut input = input;

    let data_struct = match &mut input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "ts_rs_decimal_fields can only be applied to structs",
            ));
        }
    };

    let fields = match &mut data_struct.fields {
        Fields::Named(fields) => &mut fields.named,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "ts_rs_decimal_fields only supports structs with named fields",
            ));
        }
    };

    for field in fields.iter_mut() {
        let is_decimal = match &field.ty {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "Decimal")
                .unwrap_or(false),
            _ => false,
        };

        if is_decimal {
            let already_has_ts = field.attrs.iter().any(|attr| attr.path().is_ident("ts"));
            if !already_has_ts {
                field.attrs.push(syn::parse_quote!(#[ts(type = "number")]));
            }
        }
    }

    Ok(quote!(#input))
}
