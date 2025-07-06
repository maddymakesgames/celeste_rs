use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, Meta, Type, spanned::Spanned};

enum FieldType {
    Normal(Expr, bool),
    Optional(Expr, bool),
    Child {
        is_vec: bool,
        is_optional: bool,
        is_dyn: bool,
        is_entity: bool,
    },
}

pub(super) fn map_element_derive(input: DeriveInput) -> Result<TokenStream, Error> {
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
    let mut found_child = false;
    let mut found_dyn_child = false;

    for field in &struct_data.fields {
        let mut found_rle = false;
        let mut found_attr = false;

        for attr in &field.attrs {
            if let Meta::Path(path) = &attr.meta
                && path.is_ident("rle")
            {
                found_attr = true;
                found_rle = true;
            }
        }

        for attr in &field.attrs {
            match &attr.meta {
                Meta::Path(path) =>
                    if path.is_ident("child") {
                        if found_dyn_child {
                            return Err(Error::new(
                                path.span(),
                                "dyn_child field must be the only child field",
                            ));
                        }

                        if found_rle {
                            return Err(Error::new(
                                path.span(),
                                "Can't have both rle and child on a field",
                            ));
                        }

                        found_attr = true;
                        found_child = true;

                        let is_vec = if let Type::Path(p) = &field.ty {
                            p.path.segments.first().is_some_and(|p| p.ident == "Vec")
                        } else {
                            false
                        };

                        let is_optional = if let Type::Path(p) = &field.ty {
                            p.path.segments.first().is_some_and(|p| p.ident == "Option")
                        } else {
                            false
                        };

                        fields.push((field.ident.clone().unwrap(), FieldType::Child {
                            is_vec,
                            is_optional,
                            is_dyn: false,
                            is_entity: false,
                        }));
                    } else if path.is_ident("dyn_child") {
                        if found_child {
                            return Err(Error::new(
                                path.span(),
                                "dyn_child field must be the only child field",
                            ));
                        }

                        if found_rle {
                            return Err(Error::new(
                                path.span(),
                                "Can't have both rle and child on a field",
                            ));
                        }

                        found_attr = true;
                        found_child = true;
                        found_dyn_child = true;

                        fields.push((field.ident.clone().unwrap(), FieldType::Child {
                            is_vec: true,
                            is_optional: false,
                            is_dyn: true,
                            is_entity: false,
                        }));
                    } else if path.is_ident("dyn_entities") {
                        if found_child {
                            return Err(Error::new(
                                path.span(),
                                "dyn_entities field must be the only child field",
                            ));
                        }

                        if found_rle {
                            return Err(Error::new(
                                path.span(),
                                "Can't have both rle and dyn_entities on a field",
                            ));
                        }

                        found_attr = true;
                        found_child = true;
                        found_dyn_child = true;

                        fields.push((field.ident.clone().unwrap(), FieldType::Child {
                            is_vec: true,
                            is_optional: false,
                            is_dyn: true,
                            is_entity: true,
                        }));
                    },
                Meta::NameValue(name_value) =>
                    if name_value.path.is_ident("name") {
                        found_attr = true;

                        if let Type::Path(p) = &field.ty {
                            if p.path.segments.first().is_some_and(|p| p.ident == "Option") {
                                fields.push((
                                    field.ident.clone().unwrap(),
                                    FieldType::Optional(name_value.value.clone(), found_rle),
                                ))
                            } else {
                                fields.push((
                                    field.ident.clone().unwrap(),
                                    FieldType::Normal(name_value.value.clone(), found_rle),
                                ))
                            }
                        }
                    },
                _ => continue,
            }
        }

        if !found_attr {
            return Err(Error::new(
                field.span(),
                "Field in a MapElement is missing a name, child, or dyn_child attribute",
            ));
        }
    }

    let celeste_rs = super::celeste_rs();

    let parsers = fields.iter().map(|(name, field_type)| match field_type {
        FieldType::Normal(expr, _) => quote! {#name: parser.get_attribute(#expr)?,},
        FieldType::Optional(expr, _) => quote! {#name: parser.get_optional_attribute(#expr)?,},
        FieldType::Child {
            is_vec: false,
            is_optional: true,
            is_dyn: _,
            is_entity: _,
        } => quote! {#name: parser.parse_optional_element()?,},
        FieldType::Child {
            is_vec: false,
            is_optional: false,
            is_dyn: _,
            is_entity: _,
        } => quote! {#name: parser.parse_element()?,},
        FieldType::Child {
            is_vec: true,
            is_optional: _,
            is_dyn: false,
            is_entity: _,
        } => quote! {#name: parser.parse_all_elements()?, },
        FieldType::Child {
            is_vec: true,
            is_optional: _,
            is_dyn: true,
            is_entity: false,
        } => quote! {#name: parser.parse_any_element()?, },
        FieldType::Child {
            is_vec: true,
            is_optional: _,
            is_dyn: true,
            is_entity: true,
        } => quote! {#name: parser.parse_any_entity()?, },
    });

    let encoders = fields.iter().map(|(name, field_type)| match field_type {
        FieldType::Normal(expr, false) => quote! {encoder.attribute(#expr, self.#name.clone())},
        FieldType::Normal(expr, true) => quote! {encoder.attribute(#expr, #celeste_rs::maps::EncodedVar::new_rle_str(&self.#name))},
        FieldType::Optional(expr, false) => quote! {encoder.optional_attribute(#expr, &self.#name)},
        FieldType::Optional(expr, true) => quote! {encoder.optional_attribute(#expr, &self.#name.as_ref().map(#celeste_rs::maps::EncodedVar::new_rle_str))},
        FieldType::Child{is_vec: false, is_optional: false, .. } => quote! {encoder.child(&self.#name)},
        FieldType::Child{is_vec: false, is_optional: true, .. } => quote! {if let Some(v) = &self.#name {encoder.child(v);}},
        FieldType::Child{is_vec: true, is_dyn: true, ..} => quote! {for e in &self.#name {encoder.dyn_child(e.as_ref())}},
        FieldType::Child{is_vec: true, ..} => quote! {encoder.children(&self.#name)},
    });

    Ok(quote! {
        impl MapElement for #struct_ident {
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
