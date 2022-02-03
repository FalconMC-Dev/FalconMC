use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use crate::util::properties::PropertyType;

#[derive(Debug)]
pub struct BlockData {
    base_id: i32,
    /// Keep this field as insertion order
    properties: Vec<BlockProperty>,
    base_state: BlockState,
}

impl BlockData {
    pub fn new(base_id: i32, properties: Vec<BlockProperty>, base_state: BlockState) -> Self {
        BlockData {
            base_id,
            properties,
            base_state,
        }
    }

    pub fn generate_struct_def(&self, name: &Ident) -> TokenStream {
        if self.is_empty() {
            return quote!();
        }

        let mut field_defs = Vec::new();
        let mut field_setters = Vec::new();
        let mut field_getters = Vec::new();
        for property in &self.properties {
            field_defs.push(property.generate_field_declaration());
            field_setters.push(property.generate_field_setter());
            field_getters.push(property.generate_field_getter());
        }
        let default_impl = self.base_state.generate_default_impl(name);

        quote!(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
            pub struct #name {
                #(#field_defs),*
            }

            impl #name {
                #(
                    #field_setters
                    #field_getters
                )*
            }

            #default_impl
        )
    }

    pub fn generate_property_calculation(&self, parameter: &Ident) -> TokenStream {
        let base_id = self.base_id;
        let mut sum_parts = Vec::new();
        let mut pre_calculations = Vec::new();
        let mut counter = 0;
        let mut factor = 1;
        let mut prev_count = 0;
        for (i, property) in self.properties.iter().rev().enumerate() {
            let prop_name = format_ident!("{}", property.name);
            let part = match &property.kind {
                PropertyType::Bool => quote!((!#parameter.#prop_name() as i32)),
                PropertyType::Int(_) => quote!(#parameter.#prop_name),
                PropertyType::Enum(enum_property) => enum_property.generate_calculation_part(parameter, &prop_name, &mut counter, &mut pre_calculations),
            };
            if i > 0 {
                factor *= prev_count;
                let factor_ident = syn::Index::from(factor);
                sum_parts.push(quote!( + #factor_ident * #part));
            } else {
                sum_parts.push(quote!( + #part));
            }
            prev_count = property.kind.entry_count();
        }
        quote!({
            #(#pre_calculations)*
            Some(#base_id #(#sum_parts) *)
        })
    }

    pub fn generate_from_str(&self, state_ident: &Ident, props_ident: &Ident) -> TokenStream {
        let tokens: Vec<TokenStream> = self.properties.iter()
            .map(|block_property| {
                let prop_name = block_property.name();
                let function_ident = format_ident!("with_{}", prop_name);
                let prop_type = block_property.kind.generate_type_def();
                quote!(
                    if let Some(prop) = #props_ident.get(#prop_name) {
                        #state_ident.#function_ident(#prop_type::from_str(prop)?);
                    }
                )
            }).collect();
        quote!(
            #(#tokens)*
        )
    }

    pub fn base_id(&self) -> i32 {
        self.base_id
    }

    pub fn is_empty(&self) -> bool {
        self.properties.len() == 0
    }
}

#[derive(Debug)]
pub struct BlockState {
    fields: Vec<(BlockProperty, String)>,
}

impl BlockState {
    pub fn new(fields: Vec<(BlockProperty, String)>) -> Self {
        BlockState {
            fields,
        }
    }

    pub fn generate_default_impl(&self, struct_name: &Ident) -> TokenStream {
        let field_default: Vec<TokenStream> = self.fields.iter()
            .map(|(property, raw_value)| property.generate_field_default(raw_value)).collect();
        quote!(
            impl Default for #struct_name {
                fn default() -> Self {
                    #struct_name {
                        #(#field_default),*
                    }
                }
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct BlockProperty {
    /// snake_case
    name: String,
    kind: PropertyType,
}

impl BlockProperty {
    pub fn new(name: String, kind: PropertyType) -> Self {
        BlockProperty {
            name,
            kind
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn generate_field_declaration(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let kind = self.kind.generate_type_def();
        quote!(#name: #kind)
    }

    pub fn generate_field_setter(&self) -> TokenStream {
        let setter_comment = self.kind.generate_setter_comment();
        let function_name = format_ident!("with_{}", self.name);
        let field_declaration = self.generate_field_declaration();
        let setter_logic = self.kind.generate_setter_logic(&self.name);
        quote!(
            #setter_comment
            pub fn #function_name(&mut self, #field_declaration) -> &mut Self {
                #setter_logic
                self
            }
        )
    }

    pub fn generate_field_getter(&self) -> TokenStream {
        let function_name = format_ident!("{}", self.name);
        let output_type = self.kind.generate_type_def();
        let getter_logic = self.kind.generate_getter_logic(&self.name);
        quote!(
            pub fn #function_name(&self) -> #output_type {
                #getter_logic
            }
        )
    }

    pub fn generate_field_default(&self, raw_value: &str) -> TokenStream {
        let field_name = format_ident!("{}", self.name);
        let property_value = self.kind.generate_value_tokens(raw_value);
        quote!(#field_name: #property_value)
    }
}