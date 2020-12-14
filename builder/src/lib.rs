use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Field, Ident, Type};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_ident = input.ident;
    let builder_ident = format_ident!("{}Builder", struct_ident);

    if let Data::Struct(data) = input.data {
        let fields = data
            .fields
            .iter()
            .map(|field| BuilderField::from(field))
            // Is this the right move here?
            .collect::<Vec<_>>();

        let builder_fields = fields.iter().map(|field| {
            let BuilderField { ident, ty, .. } = field;
            quote! {
                #ident: ::std::option::Option<#ty>
            }
        });

        let init_values = fields.iter().map(|field| {
            let BuilderField { ident, .. } = field;
            quote! {
                #ident: ::std::option::Option::None
            }
        });

        let setters = fields.iter().map(|field| {
            let BuilderField { ident, ty, .. } = field;
            quote! {
                fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = ::std::option::Option::Some(#ident);
                    self
                }
            }
        });

        let field_checks = fields.iter().map(|field| {
            let BuilderField {
                ident, optional, ..
            } = field;
            let message = format!(r#""{}" is required"#, ident.as_ref().unwrap());

            if !optional {
                quote! {
                    if self.#ident.is_none() {
                        return ::std::result::Result::Err(#message.into());
                    }
                }
            } else {
                quote! {}
            }
        });

        let unwrapped = fields.iter().map(|field| {
            let BuilderField {
                ident, optional, ..
            } = field;
            if !optional {
                quote! {
                    #ident: self.#ident.take().unwrap()
                }
            } else {
                quote! {
                    #ident: self.#ident.take()
                }
            }
        });

        let expanded = quote! {
            pub struct #builder_ident {
                #(#builder_fields),*
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

struct BuilderField<'a> {
    ident: Option<&'a Ident>,
    ty: &'a Type,
    optional: bool,
}

// Is there an easier way to check if a field is optional, and extract
// the inner type?
impl<'a> From<&'a Field> for BuilderField<'a> {
    fn from(field: &'a Field) -> BuilderField<'a> {
        let (ty, optional) = match &field.ty {
            Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path { segments, .. },
            }) => {
                if segments.len() == 1 {
                    match segments.first() {
                        Some(syn::PathSegment {
                            ident,
                            arguments:
                                syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments { args, .. },
                                ),
                        }) => {
                            if ident.to_string() == "Option" && args.len() == 1 {
                                match args.first() {
                                    Some(syn::GenericArgument::Type(ty)) => (ty, true),
                                    _ => (&field.ty, false),
                                }
                            } else {
                                (&field.ty, false)
                            }
                        }
                        _ => (&field.ty, false),
                    }
                } else {
                    (&field.ty, false)
                }
            }
            ty => (ty, false),
        };

        BuilderField {
            ident: field.ident.as_ref(),
            ty,
            optional,
        }
    }
}
