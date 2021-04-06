use proc_macro::TokenStream;
use quote::{quote, quote_spanned, format_ident};
use syn::{parse_macro_input, DeriveInput, Data};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    eprintln!("INPUT: {:#?}", input.data);
    let name = input.ident;
    let data = input.data;
    if let Data::Struct(ref data_struct) = *input.data {

    } else {
        // not sure
    }

    let builder_name = format_ident!("{}Builder", name);
    /*
    input.data = Struct(
    DataStruct {
        struct_token: Struct,
        fields: Named(
            FieldsNamed {
                brace_token: Brace,
                named: [
                    Field {
                        attrs: [],
                        vis: Inherited,
                        ident: Some(
                            Ident {
                                ident: "executable",
                                span: #0 bytes(377..387),
                            },
                        ),
    */
    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {

                }
            }
        }

        struct #builder_name {
            // here
        }

        impl #builder_name {

        }
    };

    proc_macro::TokenStream::from(expanded)
}


/*
use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    #[builder(each = "arg")]
    args: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .arg("build".to_owned())
        .arg("--release".to_owned())
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");
}
*/