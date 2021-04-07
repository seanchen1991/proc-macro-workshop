use quote::quote;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

fn ty_inner_type<'a>(wrapper: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper {
            return None;
        }
        
        if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments { 
            if inner_ty.args.len() != 1 { 
                return None;
            }
            
            let inner_ty = inner_ty.args.first().unwrap();
            if let syn::GenericArgument::Type(ref t) = inner_ty {
                return Some(t);
            }
        }
    }       

    None
}

fn builder_of(f: &syn::Field) -> Option<&syn::Attribute> {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "builder" {
            return Some(attr);
        }
    } 

    None
}

fn extend_method(f: &syn::Field) -> Option<(bool, proc_macro2::TokenStream)> {
    let name = f.ident.as_ref().unwrap();
    let built = builder_of(f)?;
    
    fn make_error<T: quote::ToTokens>(t: T) -> Option<(bool, proc_macro2::TokenStream)> {
        Some((false, syn::Error::new_spanned(t, "expected `builder(each = \"...\")`").to_compile_error())) 
    }

    let meta = match built.parse_meta() {
        Ok(syn::Meta::List(mut nvs)) => {
            assert!(nvs.path.is_ident("builder"));

            if nvs.nested.len() != 1 {
                return make_error(nvs);
            }

            match nvs.nested.pop().unwrap().into_value() {
                syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                    if !nv.path.is_ident("each") {
                        return make_error(nvs);
                    }

                    nv
                },
                meta => {
                    return make_error(meta);
                }
            }
        },
        Ok(meta) => {
            // inside the derive macro, there was either just an identifier i.e. `#[builder]` or a 
            // key-value pair i.e. `#[builder = "foo"]`, neither of which should work 
            return make_error(meta);
        },
        Err(e) => {
            return Some((false, e.to_compile_error()));
        }
    };

    match meta.lit {
        syn::Lit::Str(s) => {
            let arg = syn::Ident::new(&s.value(), s.span());
            let inner_ty = ty_inner_type("Vec", &f.ty).unwrap();
            let method = quote! {
                pub fn #arg(&mut self, #arg: #inner_ty) -> &mut Self {
                    self.#name.push(#arg);
                    self
                }   
            };

            Some((&arg == name, method))
        }

        lit => panic!("expected string, found {:?}", lit),
    }
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let bname = format!("{}Builder", name);
    let bident = syn::Ident::new(&bname, name.span());

    let fields = if let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }), .. }) = ast.data {
        named
    } else {
        // this derive macro not implemented on anything that isn't a struct
        unimplemented!();
    };
    
    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_inner_type("Option", ty).is_some() || builder_of(&f).is_some() {
            quote! { #name: #ty }
        } else {
            quote! { #name: std::option::Option<#ty> }
        }
    });

    let methods = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;

        let (arg_type, value) = if let Some(inner_ty) = ty_inner_type("Option", ty) {
            (inner_ty, quote! { std::option::Option::Some(#name) })
        } else if builder_of(&f).is_some() {
            (ty, quote! { #name })
        } else {
            (ty, quote! { std::option::Option::Some(#name) })
        };

        let set_method = quote! {
            pub fn #name(&mut self, #name: #arg_type) -> &mut Self {
                self.#name = #value;
                self
            }   
        };

        match extend_method(&f) {
            None => set_method.into(),
            Some((true, extended_method)) => extended_method,
            Some((false, extended_method)) => {
                let expr = quote! {
                    #set_method
                    #extended_method
                };
                expr.into()
            }
        }
    });
    
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_inner_type("Option", ty).is_some() || builder_of(f).is_some() {
            quote! { #name: self.#name.clone() }
        } else {
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
            }
        }
    });
    
    let empty_build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if builder_of(f).is_some() {
            quote! { #name: std::vec::Vec::new() }
        } else {
            quote! { #name: std::option::Option::None }
        }
    });

    let expanded = quote! {
        pub struct #bident {
            #(#builder_fields,)*
        }

        impl #bident {
            #(#methods)*

            pub fn build(&self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
                std::result::Result::Ok(#name {
                    #(#build_fields,)*
                })
            }
        }

        impl #name {
            pub fn builder() -> #bident {
                #bident {
                    #(#empty_build_fields,)*
                }
            }
        } 
    };

    expanded.into()
}
