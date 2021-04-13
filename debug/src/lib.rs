use quote::quote;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

macro_rules! return_compile_error {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => return err.to_compile_error().into()
        }
    }
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // eprintln!("{:#?}", ast);

    let ident = &ast.ident;
    let sident = syn::Ident::new(&ident.to_string(), ident.span());
    let fields = if let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }), .. }) = ast.data {
        named
    } else {
        // this derive macro not implemented for non-structs
        unimplemented!();
    };

    let debug_fields: syn::Result<Vec<_>> = fields.iter().map(|f| {
        let fname = &f.ident;
        
        // check the attrs on each field
        // if there is exactly one, check if it's the `debug` attr 
        // we care about 
        if f.attrs.len() == 1 {
            let attr = f.iter().next().unwrap();
            handle_field_attr(&f, &attr)
        } else {
            Ok(quote_spanned! {field.span()=> 
                .field(stringify!(#fname), &self.#fname)
            })
        }

        quote! { .field(stringify!(#fname), &self.#fname) }
    })
    .collect();

    let expanded = quote! {
        impl std::fmt::Debug for #sident {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmt.debug_struct(stringify!(#sident))
                #(#debug_fields)*
                .finish()
            }        
        }
    };

    expanded.into()
}
