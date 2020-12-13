use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Field};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_ident = input.ident;
    let builder_ident = format_ident!("{}Builder", struct_ident);

    if let Data::Struct(data) = input.data {
        // Here we essentially copy the fields from the original struct,
        // wrapping the types in `Option<..>`s in the process.
        let fields = data.fields.iter().map(|field| {
            let Field { ident, ty, .. } = field;
            quote! {
                #ident: Option<#ty>
            }
        });

        let init_values = data.fields.iter().map(|field| {
            let Field { ident, .. } = field;
            quote! {
                #ident: None
            }
        });

        let setters = data.fields.iter().map(|field| {
            let Field { ident, ty, .. } = field;
            quote! {
                fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            }
        });

        let expanded = quote! {
            pub struct #builder_ident {
                #(#fields),*
            }

            impl #builder_ident {
                #(#setters)*
            }

            impl #struct_ident {
                fn builder() -> #builder_ident {
                    #builder_ident {
                        #(#init_values),*
                    }
                }
            }
        };

        TokenStream::from(expanded)
    } else {
        panic!("Expected a struct")
    }
}
