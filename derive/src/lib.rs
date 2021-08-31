use heck::{CamelCase, SnakeCase};
use proc_macro::TokenStream as TStream;
use proc_macro2::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, Ident, LitStr, Token, Visibility};

struct DataConfigDeclaration {
    vis: Visibility,
    name: Ident,
    path: LitStr,
}

impl Parse for DataConfigDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token!(,)>()?;
        let path: LitStr = input.parse()?;

        Ok(DataConfigDeclaration { vis, name, path })
    }
}



#[proc_macro]
#[proc_macro_error]
pub fn data_config(input: TStream) -> TStream {
    let DataConfigDeclaration { vis, name, path } =
        parse_macro_input!(input as DataConfigDeclaration);

    let toml_file = match std::fs::read(path.value()) {
        Ok(val) => val,
        Err(err) => {
            proc_macro_error::abort!(path, "Could not read file."; note = err; note = "Make sure the file path is relative to the workspace root");
        }
    };

    let toml_config: toml::Value = match toml::from_slice(&toml_file) {
        Ok(val) => val,
        Err(err) => {
            proc_macro_error::abort!(path, "Could not read toml"; note = err);
        }
    };

    let modules = generate_modules(toml_config);

    let expanded = quote! {
        #vis mod #name {
            #(#modules)*
        }
    };

    expanded.into()
}

struct TypeDefinition {
    name: Ident,
}

struct Type {
    definition: TypeDefinition,
    builder: TokenStream,
}

fn generate_modules(toml_config: toml::Value) -> Vec<TokenStream> {
    match toml_config {
        toml::Value::Table(tbl) => tbl
            .into_iter()
            .map(|(key, val)| generate_types(key, val))
            .collect(),

        a => {
            proc_macro_error::abort_call_site!(
                "The top level toml structure needs to be a table, found {:?}",
                a.type_str()
            );
        }
    }
}

fn generate_types(name: String, toml_config: toml::Value) -> TokenStream {
    match toml_config {
        toml::Value::String(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            quote! {
                #[derive(Debug, Clone)]
                pub struct #ident(pub String);
            }
        }
        toml::Value::Integer(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            quote! {
                #[derive(Debug, Clone)]
                pub struct #ident(pub u64);
            }
        }
        toml::Value::Float(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            quote! {
                #[derive(Debug, Clone)]
                pub struct #ident(pub f64);
            }
        }
        toml::Value::Boolean(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            quote! {
                #[derive(Debug, Clone)]
                pub struct #ident(pub bool);
            }
        }
        toml::Value::Datetime(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            quote! {
                #[derive(Debug, Clone)]
                pub struct #ident(pub ::bevy_spicy_data::private::toml::Date);
            }
        }
        toml::Value::Array(_) => {
            proc_macro_error::abort_call_site!("Arrays are not supported");
        }
        toml::Value::Table(tbl) => {
            let types: Vec<TokenStream> = tbl
                .into_iter()
                .map(|(key, val)| generate_types(key, val))
                .collect();

            let ident = format_ident!("{}", name.to_snake_case());
            quote! {
                pub mod #ident {
                    #(#types)*
                }
            }
        }
    }
}
