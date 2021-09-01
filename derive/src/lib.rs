use heck::{CamelCase, SnakeCase};
use proc_macro::TokenStream as TStream;
use proc_macro2::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote, ToTokens};
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
            #modules
        }
    };

    expanded.into()
}

#[derive(Debug)]
struct TomlTypeDefinition {
    name: Ident,
    typ: TokenStream,
}

impl ToTokens for TomlTypeDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TomlTypeDefinition { name, typ } = self;
        tokens.extend(quote! {
            #[derive(::bevy_spicy_data::private::serde::Deserialize, Debug, Clone, PartialEq)]
            pub struct #name#typ
        })
    }
}

#[derive(Debug)]
struct TomlType {
    name: String,
    definition: TomlTypeDefinition,
    builder: TokenStream,
}

fn generate_modules(toml_config: toml::Value) -> TokenStream {
    match toml_config {
        toml::Value::Table(tbl) => {
            let toml_types: &Vec<TomlType> = &tbl
                .into_iter()
                .map(|(key, val)| generate_type(key, val))
                .collect();

            let types = toml_types.iter().map(|ty| {
                let TomlType {
                    name: _,
                    definition,
                    builder,
                } = ty;

                quote! {
                    #builder
                    #definition
                }
            });
            let complete_struct = toml_types.iter().map(|ty| {
                let TomlType {
                    definition:
                        TomlTypeDefinition {
                            name: ty_name,
                            typ: _typ,
                        },
                    builder: _,
                    name,
                }: &TomlType = ty;
                let field_name = format_ident!("{}", name.to_snake_case());
                quote! {
                    #[serde(rename = #name)]
                    #field_name: #ty_name
                }
            });

            quote! {
                #(#types)*

                #[derive(::bevy_spicy_data::private::serde::Deserialize, Debug, Clone, PartialEq)]
                pub struct Root {
                    #(#complete_struct),*
                }
            }
        }

        a => {
            proc_macro_error::abort_call_site!(
                "The top level toml structure needs to be a table, found {:?}",
                a.type_str()
            );
        }
    }
}

fn generate_type(name: String, toml_config: toml::Value) -> TomlType {
    match toml_config {
        toml::Value::String(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub String);},
                },
                builder: quote! {},
            }
        }
        toml::Value::Integer(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub u64);},
                },
                builder: quote! {},
            }
        }
        toml::Value::Float(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub f64);},
                },
                builder: quote! {},
            }
        }
        toml::Value::Boolean(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub bool);},
                },
                builder: quote! {},
            }
        }
        toml::Value::Datetime(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub ::bevy_spicy_data::private::toml::Date);},
                },
                builder: quote! {},
            }
        }
        toml::Value::Array(_) => {
            proc_macro_error::abort_call_site!("Arrays are not supported");
        }
        toml::Value::Table(tbl) => {
            let toml_types: &Vec<TomlType> = &tbl
                .into_iter()
                .map(|(key, val)| generate_type(key, val))
                .collect();

            let types = toml_types.iter().map(|ty| {
                let TomlType {
                    name: _,
                    definition,
                    builder,
                } = ty;

                quote! {
                    #builder
                    #definition
                }
            });

            let mod_ident = format_ident!("{}", name.to_snake_case());
            let complete_struct = toml_types.iter().map(|ty| {
                let TomlType {
                    definition:
                        TomlTypeDefinition {
                            name: ty_name,
                            typ: _,
                        },
                    builder: _,
                    name,
                }: &TomlType = ty;
                let field_name = format_ident!("{}", name.to_snake_case());
                quote! {
                    #[serde(rename = #name)]
                    #field_name: #mod_ident::#ty_name
                }
            });

            let ty_ident = format_ident!("{}", name.to_camel_case());
            TomlType {
                name,
                definition: TomlTypeDefinition {
                    name: ty_ident,
                    typ: quote! {{
                        #(#complete_struct),*
                    }},
                },
                builder: quote! {
                    pub mod #mod_ident {
                        #(#types)*
                    }
                },
            }
        }
    }
}
