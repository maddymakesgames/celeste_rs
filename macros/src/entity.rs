use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, Meta, Type, spanned::Spanned};

enum FieldType {
    Normal(Expr),
    Optional(Expr),
    Node(bool, bool),
}

pub(super) fn entity_derive(input: DeriveInput) -> Result<TokenStream, Error> {
    let Data::Struct(struct_data) = input.data.clone() else {
        unreachable!()
    };

    let struct_ident = input.ident.clone();

    let mut struct_name = None;

    for attr in &input.attrs {
        let Meta::NameValue(name_value) = &attr.meta else {
            continue;
        };

        if name_value.path.is_ident("name") {
            if !matches!(name_value.value, Expr::Lit(_)) {
                return Err(Error::new(
                    name_value.value.span(),
                    "Name attribute must have a string literal value",
                ));
            }
            struct_name = Some(name_value.value.clone());
        }
    }

    if struct_name.is_none() {
        return Err(Error::new(
            input.span(),
            "Must have a `#[name = \"\"]` attribute at the struct level",
        ));
    }

    let struct_name = struct_name.unwrap();

    let mut fields = Vec::new();

    let mut found_node = false;
    for field in &struct_data.fields {
        let mut found_attr = false;
        for attr in &field.attrs {
            match &attr.meta {
                Meta::Path(path) =>
                    if path.is_ident("node") {
                        if found_node {
                            return Err(Error::new_spanned(
                                field,
                                "An Entity implementor can't have multiple fields that take a node",
                            ));
                        }

                        found_attr = true;
                        found_node = true;


                        let is_vec = if let Type::Path(p) = &field.ty {
                            p.path.segments.first().is_some_and(|p| p.ident == "Vec")
                        } else {
                            false
                        };

                        let is_option = if let Type::Path(p) = &field.ty {
                            p.path.segments.first().is_some_and(|p| p.ident == "Option")
                        } else {
                            false
                        };

                        fields.push((
                            field.ident.clone().unwrap(),
                            FieldType::Node(is_option, is_vec),
                        ));
                    },
                Meta::NameValue(name_value) =>
                    if name_value.path.is_ident("name") {
                        found_attr = true;

                        if let Type::Path(p) = &field.ty {
                            if p.path.segments.first().is_some_and(|p| p.ident == "Option") {
                                fields.push((
                                    field.ident.clone().unwrap(),
                                    FieldType::Optional(name_value.value.clone()),
                                ))
                            } else {
                                fields.push((
                                    field.ident.clone().unwrap(),
                                    FieldType::Normal(name_value.value.clone()),
                                ))
                            }
                        }
                    },
                _ => continue,
            }
        }

        if !found_attr
            && let Type::Path(p) = &field.ty
            && p.path.segments.last().is_some_and(|p| p.ident == "Node")
        {
            found_attr = true;
            found_node = true;
            fields.push((field.ident.clone().unwrap(), FieldType::Node(false, false)));
        }

        if !found_attr {
            return Err(Error::new(
                field.span(),
                "Field in a Entity is missing a name or child attribute",
            ));
        }
    }

    let parsers = fields.iter().map(|(name, field_type)| match field_type {
        FieldType::Normal(expr) => quote! {#name: parser.get_attribute(#expr)?,},
        FieldType::Optional(expr) => quote! {#name: parser.get_optional_attribute(#expr)?,},
        FieldType::Node(true, false) => quote! {#name: parser.parse_element().ok(),},
        FieldType::Node(false, false) => quote! {#name: parser.parse_element()?,},
        FieldType::Node(_, true) => quote! {#name: parser.parse_all_elements()?, },
    });

    let encoders = fields.iter().map(|(name, field_type)| match field_type {
        FieldType::Normal(expr) => quote! {encoder.attribute(#expr, self.#name.clone())},
        FieldType::Optional(expr) => quote! {encoder.optional_attribute(#expr, &self.#name)},
        FieldType::Node(true, false) => quote! {if let Some(v) = &self.#name { encoder.child(v) }},
        FieldType::Node(false, false) => quote! {encoder.child(&self.#name)},
        FieldType::Node(_, true) => quote! {encoder.children(&self.#name)},
    });

    let celeste_rs = super::celeste_rs();

    Ok(quote! {
        impl #celeste_rs::maps::elements::entities::EntityData for #struct_ident {
            const NAME: &'static str = #struct_name;

            fn from_raw(parser: #celeste_rs::maps::parser::MapParser) -> Result<Self, #celeste_rs::maps::parser::MapElementParsingError> {
                Ok(Self {
                    #(#parsers)*
                })
            }

            fn to_raw(&self, encoder: &mut #celeste_rs::maps::encoder::MapEncoder) {
                #(#encoders;)*
            }
        }
    })
}
