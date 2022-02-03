use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::BlockData;

pub struct BlockList {
    version: i32,
    blocks: Vec<(String, BlockData)>,
}

impl BlockList {
    pub fn new(version: i32, blocks: Vec<(String, BlockData)>) -> Self {
        BlockList {
            version,
            blocks,
        }
    }

    pub fn get_block(&self, name: &str) -> Option<&BlockData> {
        self.blocks.iter().find(|(n, _)| n == name)
            .map(|(_, data)| data)
    }

    pub fn generate_enum_definition(&self) -> TokenStream {
        let mut entries = Vec::new();
        for (name, data) in &self.blocks {
            let block_name = strip_minecraft_to_pascal(&name);
            let enum_name = format_ident!("{}", block_name);
            if data.is_empty() {
                entries.push(quote!(#enum_name));
            } else {
                let state_name = format_ident!("{}State", block_name);
                entries.push(quote!(#enum_name(#state_name)));
            }
        }
        quote!(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
            pub enum Blocks {
                #(#entries),*
            }
        )
    }

    pub fn generate_block_to_id_definition(&self, reference: Option<&BlockList>) -> TokenStream {
        let function_ident = format_ident!("get_global_id_{}", self.version as usize);
        let match_entries: Vec<TokenStream> = self.blocks.iter()
            .map(|(name, data)| {
                let block_name = strip_minecraft_to_pascal(name);
                let enum_entry_name = format_ident!("{}", block_name);
                let base_id = data.base_id();
                if data.is_empty() {
                    match reference.map(|x| x.get_block(name)) {
                        Some(Some(data)) if !data.is_empty() => quote!(Blocks::#enum_entry_name(_) => Some(#base_id)),
                        _ => quote!(Blocks::#enum_entry_name => Some(#base_id)),
                    }
                } else {
                    let state_ident = format_ident!("state_unique_falcon");
                    let state_result = data.generate_property_calculation(&state_ident);
                    quote!(Blocks::#enum_entry_name(#state_ident) => #state_result)
                }
            }).collect();
        let end_part = match reference {
            Some(_) => quote!(_ => None,),
            None => quote!()
        };

        quote!(
            pub fn #function_ident(&self) -> Option<i32> {
                match self {
                    #(#match_entries,)*
                    #end_part
                }
            }
        )
    }

    pub fn generate_struct_definitions(&self) -> TokenStream {
        println!("Version: {}", self.version);
        let definitions: Vec<TokenStream> = self.blocks.iter()
            .filter(|(_, data)| !data.is_empty())
            .map(|(name, data)| {
                let block_name = strip_minecraft_to_pascal(name);
                let struct_name = format_ident!("{}State", block_name);
                data.generate_struct_def(&struct_name)
        }).collect();
        quote!(
            #(#definitions)*
        )
    }

    pub fn generate_from_str_impl(&self) -> TokenStream {
        let properties_ident = format_ident!("props");
        let state_ident = format_ident!("state_ident");
        let branches: Vec<TokenStream> = self.blocks.iter()
            .map(|(name, data)| {
                let block_name = strip_minecraft(name);
                let enum_name = format_ident!("{}", strip_minecraft_to_pascal(name));
                if data.is_empty() {
                    quote!(#block_name => Blocks::#enum_name)
                } else {
                    let state_name = format_ident!("{}State", enum_name);
                    let property_building = data.generate_from_str(&state_ident, &properties_ident);
                    quote!(
                        #block_name => {
                            let mut #state_ident = #state_name::default();
                            #property_building
                            Blocks::#enum_name(#state_ident)
                        }
                    )
                }
            }).collect();

        quote!(
            impl std::str::FromStr for Blocks {
                type Err = ParseBlockError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let (_domain, stripped) = if s.contains(':') { s.split_once(':').unwrap() } else { ("", s) };
                    let (name, stripped) = if let Some(i) = stripped.rfind(']') {
                        let _ = stripped.find('[').ok_or(ParseBlockError::InvalidToken)?;
                        stripped.split_at(i).0.split_once('[').unwrap()
                    } else {
                        (stripped, "")
                    };
                    let #properties_ident: ::ahash::AHashMap<&str, &str> = stripped.split(',')
                        .map(|x| x.split_once('=')).flatten().collect();
                    Ok(match name {
                        #(#branches,)*
                        _ => return Err(ParseBlockError::UnknownBlock),
                    })
                }
            }
        )
    }
}

fn strip_minecraft_to_pascal(name: &str) -> String {
    strip_minecraft(name).to_case(Case::Pascal)
}

fn strip_minecraft(name: &str) -> &str {
    name.split_once(":").unwrap().1
}