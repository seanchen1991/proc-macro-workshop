use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned,
    parse_macro_input, 
    DeriveInput
};

macro_rules! return_compile_error {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => return err.to_compile_error().into()
        }
    }
}

fn handle_field_attr(f: &syn::Field, attr: &syn::Attribute) -> syn::Result<proc_macro2::TokenStream> {
    let fname = &f.ident;
    let meta = syn::Attribute::parse_meta(attr)?;

    if let syn::Meta::NameValue(syn::MetaNameValue {
        path,
        lit: syn::Lit::Str(lit),
        ..
    }) = &meta {
        if path.is_ident("debug") {
            return Ok(quote_spanned! {attr.span()=>
                .field(stringify!(#fname), &format_args!(#lit, &self.#fname))
            });
        } 
    }

    Err(syn::Error::new_spanned(&meta, "expected `debug = \"...\"`"))
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
            let attr = f.attrs.iter().next().unwrap();
            handle_field_attr(&f, &attr)
        } else {
            Ok(quote_spanned! {f.span()=> 
                .field(stringify!(#fname), &self.#fname)
            })
        }
    })
    .collect();

    let debug_fields = return_compile_error!(debug_fields);

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
