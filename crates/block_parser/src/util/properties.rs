use std::ops::Range;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyType {
    Bool,
    Int(Range<u8>),
    Enum(EnumProperty),
}

impl PropertyType {
    pub fn entry_count(&self) -> usize {
        match self {
            PropertyType::Bool => 2,
            PropertyType::Int(range) => (range.end - range.start) as usize,
            PropertyType::Enum(property) => property.entry_count(),
        }
    }

    pub fn generate_type_def(&self) -> TokenStream {
        let type_name = match self {
            PropertyType::Bool => syn::Path::from(format_ident!("{}", "bool")),
            PropertyType::Int(_) => syn::Path::from(format_ident!("{}", "i32")),
            PropertyType::Enum(v) => syn::Path::from(format_ident!("{}", v.name)),
        };
        quote!(#type_name)
    }

    pub fn generate_setter_comment(&self) -> TokenStream {
        if let PropertyType::Int(range) = self {
            let comment1 = format!("This is a value between {} and {} (both ends inclusive).", range.start, range.end - 1);
            let comment2 = format!("Developers should be careful to respect these bounds as no checking is done at runtime!!!");
            return quote!(
                #[doc = #comment1]
                #[doc = #comment2]
            );
        }
        quote!()
    }

    pub fn generate_setter_logic(&self, field_name: &str) -> TokenStream {
        let field_name = format_ident!("{}", field_name);
        match self {
            PropertyType::Int(range) if range.start != 0 => {
                let start = syn::Index::from(range.start as usize);
                quote!(self.#field_name = #field_name - #start;)
            }
            _ => quote!(self.#field_name = #field_name;),
        }
    }

    pub fn generate_getter_logic(&self, field_name: &String) -> TokenStream {
        let field_name = format_ident!("{}", field_name);
        match self {
            PropertyType::Int(range) if range.start != 0 => {
                let start = syn::Index::from(range.start as usize);
                quote!(self.#field_name + #start)
            }
            _ => quote!(self.#field_name),
        }
    }

    pub fn generate_value_tokens(&self, raw_value: &str) -> TokenStream {
        match self {
            PropertyType::Bool => {
                let value = raw_value.parse::<bool>().expect(&format!("Unexpected value for bool: {}", raw_value));
                quote!(#value)
            }
            PropertyType::Int(range) => {
                let value = raw_value.parse::<i32>().expect(&format!("Unexpected value for i32: {}", raw_value)) - (range.start as i32);
                quote!(#value)
            }
            PropertyType::Enum(v) => {
                if !v.fields.contains(&String::from(raw_value)) {
                    panic!("Unexpected value for {}: {}", v.name, raw_value)
                }
                let value = format_ident!("{}", raw_value.to_case(Case::Pascal));
                let enum_name = format_ident!("{}", v.name);
                quote!(#enum_name::#value)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumProperty {
    /// PascalCase
    name: String,
    /// snake_case
    fields: Vec<String>,
    default_order: Option<Vec<String>>,
}

impl EnumProperty {
    pub fn new<T: Into<String>>(name: T, mut fields: Vec<T>) -> Self {
        EnumProperty {
            name: name.into(),
            fields: fields.drain(..).map(|x| x.into()).collect(),
            default_order: None,
        }
    }

    pub fn with_order(name: String, fields: Vec<String>, default_order: Vec<String>) -> Self {
        EnumProperty {
            name,
            fields,
            default_order: Some(default_order),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.fields.get(index)
    }

    pub fn entry_count(&self) -> usize {
        self.fields.len()
    }

    pub fn fields_cloned(&self) -> Vec<String> {
        self.fields.clone()
    }

    pub fn generate_enum_definition(&self) -> TokenStream {
        let enum_name = format_ident!("{}", self.name);
        let enum_fields: Vec<Ident> = self.fields.iter()
            .map(|field| format_ident!("{}", field.to_case(Case::Pascal))).collect();
        quote!(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
            pub enum #enum_name {
                #(#enum_fields),*
            }
        )
    }

    pub fn generate_from_str_impl(&self) -> TokenStream {
        let enum_name = format_ident!("{}", self.name);
        let enum_values: Vec<TokenStream> = self.fields.iter()
            .map(|field| {
                let value_name = format_ident!("{}", field.to_case(Case::Pascal));
                quote!(#field => Ok(#enum_name::#value_name))
            }).collect();
        quote!(
            impl std::str::FromStr for #enum_name {
                type Err = ParseBlockError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        #(#enum_values,)*
                        _ => Err(ParseBlockError::InvalidProperty),
                    }
                }
            }
        )
    }

    pub fn generate_calculation_part(&self, parameter: &Ident, property_name: &Ident, counter: &mut usize, pre_calculations: &mut Vec<TokenStream>) -> TokenStream {
        match &self.default_order {
            Some(_) => {
                let field_branches: Vec<TokenStream> = self.fields.iter().enumerate()
                    .map(|(i, field)| {
                        let enum_type = format_ident!("{}", self.name);
                        let variant_name = format_ident!("{}", field.to_case(Case::Pascal));
                        let number_ident = syn::Index::from(i);
                        quote!(#enum_type::#variant_name => #number_ident)
                    }).collect();
                let variable_name = format_ident!("temp_var_local{}", *counter);
                pre_calculations.push(quote!(
                    let #variable_name = match #parameter.#property_name {
                        #(#field_branches,)*
                        _ => return None,
                    };
                ));
                *counter += 1;
                quote!(#variable_name)
            }
            None => quote!((#parameter.#property_name as i32)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumPropertyBase {
    property_set: Vec<EnumProperty>,
}

impl EnumPropertyBase {
    pub fn new(properties: Vec<EnumProperty>) -> Self {
        EnumPropertyBase {
            property_set: properties,
        }
    }

    pub fn find_property(&self, name: &str, values: &Vec<String>) -> Option<&EnumProperty> {
        self.property_set.iter().find(|prop| prop.name.eq(name) || prop.fields.eq(values))
    }

    pub fn find_fields(&self, values: &Vec<String>) -> Option<&EnumProperty> {
        self.property_set.iter().find(|prop| values.iter().all(|x| prop.fields.contains(x)))
    }

    pub fn add(&mut self, property: EnumProperty) -> EnumProperty {
        self.property_set.push(property.clone());
        property
    }

    pub fn generate_enum_list(&self) -> TokenStream {
        let mut enum_definitions = Vec::new();
        let mut enum_from_str_impls = Vec::new();
        for property in &self.property_set {
            enum_definitions.push(property.generate_enum_definition());
            enum_from_str_impls.push(property.generate_from_str_impl());
        }
        quote!(
            #(
                #enum_definitions
                #enum_from_str_impls
            )*
        )
    }
}