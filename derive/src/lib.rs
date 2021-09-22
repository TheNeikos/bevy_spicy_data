use heck::{CamelCase, SnakeCase};
use proc_macro::TokenStream as TStream;
use proc_macro2::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, LitStr, Token, Visibility, parse::Parse, parse_macro_input};

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

            let uuid = uuid::Uuid::new_v4().as_bytes().to_vec();

            let type_register = toml_types.iter().map(|ty| {
                let TomlType { name, definition: TomlTypeDefinition { name: ty_name, typ: _ }, builder: _ } = ty;

                let field_name = format_ident!("{}", name.to_snake_case());
                quote! {
                    <#ty_name as ::bevy_spicy_data::Config>::register(&self.#field_name, load_context, Some(vec![String::from(#name)]));
                }                
            });

            let child_assets = toml_types.iter().map(|ty| {
                let TomlType { name: _, definition: TomlTypeDefinition { name: ty_name, typ: _ }, builder: _ } = ty;

                quote! {
                    <#ty_name as ::bevy_spicy_data::Config>::add_asset(app);
                }                
            });

            quote! {
                #(#types)*

                #[derive(::bevy_spicy_data::private::serde::Deserialize, Debug, Clone, PartialEq)]
                pub struct Root {
                    #(#complete_struct),*
                }

                impl ::bevy_spicy_data::Config for Root {
                    fn register<'a>(&self, load_context: &'a mut ::bevy_spicy_data::private::LoadContext, _path: Option<Vec<String>>) {
                        load_context.set_default_asset(::bevy_spicy_data::private::LoadedAsset::new(<Root as Clone>::clone(self)));

                        #(#type_register)*
                    }

                    fn add_asset(app: &mut ::bevy_spicy_data::private::App) {
                        use ::bevy_spicy_data::private::AddAsset;

                        app.add_asset::<Self>();

                        #(#child_assets)*
                    }
                }

                impl ::bevy_spicy_data::private::TypeUuid for Root {
                    const TYPE_UUID: ::bevy_spicy_data::private::Uuid = ::bevy_spicy_data::private::Uuid::from_bytes([#(#uuid),*]);
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

fn make_builder(ty_name: &Ident, children: Option<(Vec<TokenStream>, Vec<TokenStream>)>, custom_add_asset: Option<TokenStream>) -> TokenStream {
    let uuid = uuid::Uuid::new_v4().as_bytes().to_vec();
    let (register, add_asset) = if let Some((register, add_asset)) = children {
        (register, add_asset)
    } else {
        (vec![], vec![])
    };

    let custom_add_asset = custom_add_asset.unwrap_or_default();

    quote! {
        impl ::bevy_spicy_data::Config for #ty_name {
            fn register<'a>(&self, load_context: &'a mut ::bevy_spicy_data::private::LoadContext, path: Option<Vec<String>>) {
                let asset_path = path.as_ref().unwrap().join(".");
                load_context.set_labeled_asset(&asset_path, ::bevy_spicy_data::private::LoadedAsset::new(<Self as Clone>::clone(self)));

                #(#register)*
            }


            fn add_asset(app: &mut ::bevy_spicy_data::private::App) {
                use ::bevy_spicy_data::private::AddAsset;
                
                app.add_asset::<Self>();

                #(#add_asset)*

                #custom_add_asset
            }
        }

        impl ::bevy_spicy_data::private::TypeUuid for #ty_name {
            const TYPE_UUID: ::bevy_spicy_data::private::Uuid = ::bevy_spicy_data::private::Uuid::from_bytes([#(#uuid),*]);
        }
    }
}

fn generate_type(name: String, toml_config: toml::Value) -> TomlType {


    match toml_config {
        toml::Value::String(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            let builder = make_builder(&ident, None, Some(quote! {
                app.add_system_to_stage(::bevy_spicy_data::SyncStage, ::bevy_spicy_data::UiDataText::<Self>::when_inserted);
                app.add_system_to_stage(::bevy_spicy_data::SyncStage, ::bevy_spicy_data::UiDataText::<Self>::keep_in_sync);
            }));

            TomlType {
                name,
                builder: quote! {
                    #builder

                    impl ::std::convert::AsRef<str> for #ident {
                        fn as_ref(&self) -> &str {
                            &self.0
                        }
                    }
                },
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub String);},
                },
            }
        }
        toml::Value::Integer(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                builder: make_builder(&ident, None, None),
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub u64);},
                },
            }
        }
        toml::Value::Float(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                builder: make_builder(&ident, None, None),
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub f64);},
                },
            }
        }
        toml::Value::Boolean(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                builder: make_builder(&ident, None, None),
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub bool);},
                },
            }
        }
        toml::Value::Datetime(_) => {
            let ident = format_ident!("{}", name.to_camel_case());

            TomlType {
                name,
                builder: make_builder(&ident, None, None),
                definition: TomlTypeDefinition {
                    name: ident,
                    typ: quote! {(pub ::bevy_spicy_data::private::toml::Date);},
                },
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
            let config_builder = make_builder(&ty_ident, Some((toml_types.iter().map(|ty| {
                let TomlType { name: child_name, definition: TomlTypeDefinition { name: ty_name, typ: _ }, builder: _ } = ty;

                let field_name = format_ident!("{}", child_name.to_snake_case());
                quote! {
                    <#mod_ident::#ty_name as ::bevy_spicy_data::Config>::register(&self.#field_name, load_context, Some({
                        let mut path: Vec<String> = path.as_ref().unwrap().clone();
                        path.push(String::from(#child_name));
                        path
                    }));
                }
            }).collect(),toml_types.iter().map(|ty| {
                let TomlType { name: _child_name, definition: TomlTypeDefinition { name: ty_name, typ: _ }, builder: _ } = ty;

                quote! {
                    <#mod_ident::#ty_name as ::bevy_spicy_data::Config>::add_asset(app);
                }
            }).collect(),
            )), None);
            TomlType {
                name,
                definition: TomlTypeDefinition {
                    name: ty_ident,
                    typ: quote! {{
                        #(#complete_struct),*
                    }},
                },
                builder: quote! {
                    #config_builder

                    pub mod #mod_ident {
                        #(#types)*
                    }
                },
            }
        }
    }
}
