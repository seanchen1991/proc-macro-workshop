use quote::quote;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

fn debug_attr(f: &syn::Field) -> Option<&syn::Attribute> {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "debug" {
            return Some(attr);
        }
    }

    None
}

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    eprintln!("{:#?}", ast);

    let ident = &ast.ident;
    let sident = syn::Ident::new(&ident.to_string(), ident.span());
    let fields = if let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }), .. }) = ast.data {
        named
    } else {
        // this derive macro not implemented for non-structs
        unimplemented!();
    };

    let _struct_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: #ty }
    });

    let debug_methods = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { .field(stringify!(#name), &self.#name) }
    });

    let expanded = quote! {
        // pub struct #sident {
        //     #(#struct_fields,)*
        // }

        impl std::fmt::Debug for #sident {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmt.debug_struct(stringify!(#sident))
                #(#debug_methods)*
                .finish()
            }        
        }
    };

    expanded.into()
}
