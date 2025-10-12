use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(PartialStruct)]
pub fn derive_optional_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let vis = input.vis;
    let opt_name = format_ident!("Partial{}", name);

    let data = match input.data {
        Data::Struct(s) => s,
        _ => panic!("PartialStruct can only be used on structs"),
    };

    let mut opt_fields = Vec::new();
    let mut update_fields = Vec::new();

    if let Fields::Named(fields) = data.fields {
        for field in fields.named {
            let ident = field.ident.unwrap();
            let ty = field.ty;

            // Optional field type
            let opt_ty = quote! { Option<#ty> };

            opt_fields.push(quote! {
                #[serde(default)]
                #ident: #opt_ty
            });

            update_fields.push(quote! {
                if let Some(v) = self.#ident {
                    base.#ident = v;
                }
            });
        }
    } else {
        panic!("PartialStruct only supports named structs");
    }

    let expanded = quote! {
        #[derive(::serde::Deserialize, ::std::default::Default, ::std::clone::Clone, ::std::fmt::Debug)]
        #vis struct #opt_name {
            #(#opt_fields,)*
        }

        impl #opt_name {
            pub fn apply_to(self, base: &mut #name) {
                #(#update_fields)*
            }
        }
    };

    TokenStream::from(expanded)
}
