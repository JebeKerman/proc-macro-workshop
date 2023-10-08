use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let data = match input.data {
        syn::Data::Struct(s) => s,
        syn::Data::Enum(_) | syn::Data::Union(_) => panic!("Derive Custom Debug is only implemented for structs"),
    };

    let mut debug_fields = quote!();
    let len = data.fields.len();
    for (idx, field) in data.fields.iter().enumerate() {
        if let Some(name) = &field.ident {
            let mut debug_fmt = None;
            for attr in  &field.attrs {
                match &attr.meta {
                    syn::Meta::Path(_) | syn::Meta::List(_) => panic!("Only NameValue attributes are imlemented"),
                    syn::Meta::NameValue(val) => {
                        if val.path.is_ident("debug") {
                            debug_fmt = Some(&val.value);
                        } else {
                            panic!("unkown attribute {}", val.path.to_token_stream());
                        }
                    },
                }
            }

            let mut fmt = if let Some(fmt) = debug_fmt {
                format!("{{}}: {}", fmt.to_token_stream().to_string().trim_end_matches("\"").trim_start_matches("\""))
            } else {
                String::from("{}: {:?}")
            };
            if idx + 1 < len {
                fmt.push(',');
            }
            fmt.push(' ');

            debug_fields.extend(quote!(
                write!(f, #fmt, stringify!(#name), &self.#name)?;
            ))  
        }
    };

    //field("name", &self.name).field("bitmask", &self.bitmask)

    let result = quote!(
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {{ ", stringify!(#name))?;
                #debug_fields
                write!(f, "}}")
            }
        }
    );
   
    TokenStream::from(result)
}
