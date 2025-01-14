use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Filterable, attributes(filter))]
pub fn filterable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    // Extract fields marked with #[filter]
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let filter_fields = fields
        .iter()
        .filter(|field| {
            field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("filter"))
        })
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            (field_name, field_type)
        })
        .collect::<Vec<_>>();

    // Generate FilterCondition enum variants
    let conditions = filter_fields.iter().map(|(name, ty)| {
        let variant_name = format_ident!("{}", name.to_string().to_uppercase());
        quote! {
            #variant_name(#ty)
        }
    });

    // Generate matches arm for the matches() implementation
    let match_arms = filter_fields.iter().map(|(name, _)| {
        let variant_name = format_ident!("{}", name.to_string().to_uppercase());
        quote! {
            FilterCondition::#variant_name(val) => self.#name == *val
        }
    });

    let expanded = quote! {
        #[derive(Clone)]
        pub enum FilterCondition {
            #(#conditions),*
        }

        impl Filterable for #name {
            fn matches(&self, filter: &Filter) -> bool {
                if filter.conditions.is_empty() {
                    return true;
                }

                filter.conditions.iter().all(|condition| {
                    match condition {
                        #(#match_arms),*
                    }
                })
            }
        }
    };

    TokenStream::from(expanded)
}
