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
                #ident: ::std::option::Option<#ty>
            }
        });

        let init_values = data.fields.iter().map(|field| {
            let Field { ident, .. } = field;
            quote! {
                #ident: ::std::option::Option::None
            }
        });

        let setters = data.fields.iter().map(|field| {
            let Field { ident, ty, .. } = field;
            quote! {
                fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = ::std::option::Option::Some(#ident);
                    self
                }
            }
        });

        let field_checks = data.fields.iter().map(|field| {
            let Field { ident, .. } = field;
            let message = format!(r#""{}" is required"#, ident.as_ref().unwrap());

            quote! {
                if self.#ident.is_none() {
                    return ::std::result::Result::Err(#message.into());
                }
            }
        });

        let unwrapped = data.fields.iter().map(|field| {
            let Field { ident, .. } = field;
            quote! {
                #ident: self.#ident.take().unwrap()
            }
        });

        let expanded = quote! {
            pub struct #builder_ident {
                #(#fields),*
            }

            impl #builder_ident {
                #(#setters)*

                fn build(&mut self) -> Result<#struct_ident, Box<dyn ::std::error::Error>> {
                    #(#field_checks)*

                    Ok(#struct_ident {
                        #(#unwrapped),*
                    })
                }
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
