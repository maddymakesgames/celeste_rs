use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Error, Expr, ExprLit, Fields, Lit, Meta};

pub(super) fn yaml_derive_enum(
    input: DeriveInput,
    enum_data: DataEnum,
) -> Result<TokenStream, Error> {
    let enum_name = input.ident;
    let mut variants = Vec::with_capacity(enum_data.variants.len());
    let mut unit_variants = Vec::with_capacity(enum_data.variants.len());
    for variant in enum_data.variants {
        if let Fields::Unit = variant.fields {
            unit_variants.push(variant);
        } else {
            variants.push(variant);
        }
    }

    variants.sort_by(|a, b| a.fields.len().cmp(&b.fields.len()));

    let mut parsers = Vec::with_capacity(variants.len() + 1);
    let mut writer_branches = Vec::with_capacity(variants.len() + unit_variants.len());

    let unit_branches = unit_variants
        .iter()
        .map(|v| {
            let mut name_str = v.ident.to_string();

            for attr in &v.attrs {
                if let Meta::NameValue(mnv) = &attr.meta {
                    if mnv.path.is_ident("rename") {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) = &mnv.value
                        {
                            name_str = lit_str.value();
                            // We don't want to allow mutliple renames
                            break;
                        }
                    }
                }
            }

            let name = &v.ident;
            quote! {
                #name_str => #enum_name::#name
            }
        })
        .collect::<Vec<_>>();

    let celeste_rs = crate::celeste_rs();
    let enum_name_str = enum_name.to_string();
    parsers.push(quote! {
        if let Ok(string) = yaml.try_as_str() {
            return Ok(match string {
                #(#unit_branches,)*
                _ => return Err(YamlParseError::UnknownVariant(#enum_name_str, string.to_owned()))
            })
        }
    });

    for variant in unit_variants {
        let name = &variant.ident;
        let name_str = &variant.ident.to_string();
        writer_branches.push(quote! {
            #enum_name::#name => Yaml::string(#name_str.to_owned())
        });
    }

    #[allow(clippy::never_loop)]
    for variant in variants {
        // TODO: actually support enums with fields
        // this is a lot more work and isn't immediately useful so I'm putting it off
        match variant.fields {
            Fields::Unit => unreachable!("we already separated unit branches"),
            Fields::Named(_) | Fields::Unnamed(_) =>
                return Err(Error::new(
                    enum_name.span(),
                    "FromYaml can only be derived on enums with only unit variants",
                )),
        }
    }

    Ok(quote! {
        impl #celeste_rs::utils::yaml::FromYaml for #enum_name {
            fn parse_from_yaml(yaml: &#celeste_rs::utils::yaml::saphyr::Yaml) -> Result<Self, #celeste_rs::utils::yaml::YamlParseError> {
                #[allow(unused_imports)]
                use #celeste_rs::utils::yaml::{FromYaml, HashExt, YamlExt, YamlParseError, YamlWriteError, saphyr::{Yaml, Scalar, Mapping}};
                #(#parsers)*

                Err(YamlParseError::Custom(format!("Could not find a matching '{}' variant when parsing", #enum_name_str)))
            }

            fn to_yaml(&self) -> Result<#celeste_rs::utils::yaml::saphyr::Yaml, #celeste_rs::utils::yaml::YamlWriteError> {
                #[allow(unused_imports)]
                use #celeste_rs::utils::yaml::{FromYaml, HashExt, YamlExt, YamlParseError, YamlWriteError, saphyr::{Yaml, Scalar, Mapping}};
                Ok(match self {
                    #(#writer_branches,)*
                    _ => return Err(YamlWriteError::Custom("Can't serialize non-unit enum variants right now".to_owned()))
                })
            }
        }
    })
}
