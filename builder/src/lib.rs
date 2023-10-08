use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, parse_macro_input, DeriveInput, PathArguments};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let buildername = Ident::new(&format!("{}Builder", name.to_string()), name.span().clone());

    let data = match input.data {
        syn::Data::Struct(s) => s,
        syn::Data::Enum(_) | syn::Data::Union(_) => panic!("Dont handle something like that.. jesus")
    };

    
    let mut field_names = quote!();
    let mut builder_fields = quote!();
    let mut init_fields_none = quote!();
    let mut field_set_vec = quote!();
    let mut builder_build = quote!();

    match data.fields {
        syn::Fields::Named(fields) => {
            for field in fields.named {
                let name = field.ident.unwrap();
                let ty = field.ty;

                let mut inner = None;
                let mut is_option = false; 
                if let syn::Type::Path(path) = &ty {
                    
                    let segments = &path.path.segments;
                    if let Some(first) = segments.first() {
                        if first.ident == "Option" {
                            match &first.arguments {
                                PathArguments::None => eprintln!("none"),
                                PathArguments::AngleBracketed(innert) => {
                                    inner = Some(innert.args.clone());
                                },
                                PathArguments::Parenthesized(_) => eprintln!("paraent"),
                            };
                            eprintln!("-------------------------- {} {:?}", path.to_token_stream(), first.arguments.to_token_stream());
                            is_option = true;
                        }
                    } else {
                        panic!("idk what to do");
                    }
                } else {
                    panic!("idk what to do");
                }


                field_names.extend(quote!(
                    #name,
                ));

                init_fields_none.extend(quote!{
                    #name: None,
                });

                builder_fields.extend(
                    if is_option {
                        quote!{
                            #name: #ty,
                    }} else {
                        quote!{
                            #name: Option<#ty>,
                        }
                });

                builder_build.extend(
                    if is_option {
                        quote! {
                            let #name = self.#name.clone();
                        }
                    } else {
                        quote! {
                            let #name = self.#name.clone().ok_or(format!("{} is none but must be set", stringify!(#name)))?;
                        }
                    }
                );
    

                field_set_vec.extend(
                    if is_option {
                        let inner = inner.unwrap();
                        quote! {
                            fn #name(&mut self, #name: #inner) -> &mut Self {
                                self.#name = Some(#name);
                                self
                            }
                        }
                    } else {
                        quote!(
                            fn #name(&mut self, #name: #ty) -> &mut Self {
                                self.#name = Some(#name);
                                self
                            }
                        )
                    }
                );
            }                    
        },
        _ => unimplemented!()
    }

    let expanded = quote! {
        pub struct #buildername {
            #builder_fields
        }

        impl #buildername {
            #field_set_vec

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                #builder_build

                Ok(#name { #field_names })
            }  
        }
        
        impl #name {
            pub fn builder() -> #buildername {
                #buildername {
                    #init_fields_none
                }
            }
        }
    };

    TokenStream::from(expanded)
}
