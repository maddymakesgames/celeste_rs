use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    spanned::Spanned,
    Data,
    DeriveInput,
    Error,
    Expr,
    ExprLit,
    GenericArgument,
    Ident,
    Lit,
    Meta,
    PathArguments,
    Type,
    TypeArray,
    TypePath,
};

use crate::celeste_rs;

pub(super) fn yaml_derive(input: DeriveInput) -> Result<TokenStream, Error> {
    let Data::Struct(struct_data) = input.data.clone() else {
        unreachable!()
    };

    let struct_ident = input.ident.clone();

    let mut fields = Vec::new();

    for field in &struct_data.fields {
        if field.ident.is_none() {
            return Err(Error::new(
                input.span(),
                "YamlFile derive cannot currently be implemented on tuple structs",
            ));
        }

        let mut name = field.ident.clone().unwrap().to_string();
        let mut parsing_fn = None;
        let mut writing_fn = None;

        for attr in &field.attrs {
            if let Meta::NameValue(name_value) = &attr.meta {
                if name_value.path.is_ident("name") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }) = &name_value.value
                    {
                        name = str.value();
                    } else {
                        return Err(Error::new(
                            name_value.value.span(),
                            "name attribute needs string literal value",
                        ));
                    }
                } else if name_value.path.is_ident("parse_fn") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }) = &name_value.value
                    {
                        parsing_fn = Some(Ident::new(&str.value(), str.span()));
                    }
                } else if name_value.path.is_ident("write_fn") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }) = &name_value.value
                    {
                        writing_fn = Some(Ident::new(&str.value(), str.span()));
                    }
                }
            }
        }

        fields.push(Field {
            rust_name: field.ident.clone().unwrap(),
            yaml_name: name,
            rust_type: field.ty.clone(),
            parsing_fn,
            writing_fn,
        });
    }

    let celeste_rs = crate::celeste_rs();

    let field_parsers = fields
        .iter()
        .map(gen_field_parser)
        .collect::<Result<Vec<_>, _>>()?;
    let field_names = fields
        .iter()
        .map(|f| f.rust_name.clone())
        .collect::<Vec<_>>();
    let field_writers = fields
        .iter()
        .map(gen_field_writer)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {

            impl #celeste_rs::utils::yaml::YamlFile for #struct_ident {
                fn parse_from_yaml(yaml: &#celeste_rs::utils::yaml::saphyr::Yaml) -> Result<Self, #celeste_rs::utils::yaml::YamlParseError> {
                    #(#field_parsers)*

                    Ok(#struct_ident {
                        #(#field_names),*
                    })
                }

                fn to_yaml(&self) -> Result<#celeste_rs::utils::yaml::saphyr::Yaml, #celeste_rs::utils::yaml::YamlWriteError> {
                    let mut output = saphyr::Hash::new();

                    #(#field_writers);*

                    Ok(#celeste_rs::utils::yaml::saphyr::Yaml::Hash(output))
                }
            }

    })
}

fn gen_field_parser(field: &Field) -> Result<TokenStream, Error> {
    if let Some(func) = &field.parsing_fn {
        let ident = &field.rust_name;
        return Ok(quote! {
            let #ident = #func(&yaml)?;
        });
    }

    let parser = gen_type_parse(&field.yaml_name, &field.rust_type)?;
    let ident = &field.rust_name;
    Ok(quote! {
        let #ident = {#parser};
    })
}

fn gen_type_parse(name: &str, ty: &Type) -> Result<TokenStream, Error> {
    match &ty {
        Type::Array(TypeArray { elem, len, .. }) => gen_array_parser(name, elem, len),
        Type::Paren(type_paren) => gen_type_parse(name, &type_paren.elem),
        Type::Path(_) => gen_path_parser(name, ty),
        t => Err(Error::new(
            t.span(),
            format!("Field of type {t:?} is not allowed in YamlFile derive"),
        ))?,
    }
}

fn gen_path_parser(name: &str, ty: &Type) -> Result<TokenStream, Error> {
    let celeste_rs = celeste_rs();

    Ok(if let Some(ty) = get_option_ty(ty) {
        quote! {<#ty as #celeste_rs::utils::yaml::YamlFile>::parse_from_yaml(&yaml[#name]).ok()}
    } else {
        quote! {<#ty as #celeste_rs::utils::yaml::YamlFile>::parse_from_yaml(&yaml[#name])?}
    })
}

fn gen_array_parser(name: &str, ty: &Type, len: &Expr) -> Result<TokenStream, Error> {
    let celeste_rs = celeste_rs();
    let is_option = is_option_type(ty);
    let output_dec = if is_option {
        quote! {[None; #len]}
    } else {
        quote! {[std::mem::MaybeUninit::uninit(); #len]}
    };

    let header = quote! {
        let arr = yaml[#name].as_vec()
            .ok_or(#celeste_rs::utils::yaml::YamlParseError::TypeMismatch(
                "Vec",
                #celeste_rs::utils::yaml::yaml_type_name(&yaml[#name])
            ))?;
        let mut output = #output_dec;
        let mut i = 0;
    };


    let len_check = if is_option {
        quote! {}
    } else {
        quote! {
            if arr.len() != #len {
                return Err(#celeste_rs::utils::yaml::YamlParseError::ArrayLenMismatch(#name, arr.len(), #len));
            }
        }
    };


    let parser = if is_option {
        // we already checked that we had an option, and an option needs generic args
        let ty = get_option_ty(ty).ok_or(Error::new(
            ty.span(),
            "Invalid Option, can't find inner type",
        ))?;

        quote! {
            output[i] = <#ty as #celeste_rs::utils::yaml::YamlFile>::parse_from_yaml(ele).ok();
        }
    } else {
        quote! {
            output[i].write(<#ty as #celeste_rs::utils::yaml::YamlFile>::parse_from_yaml(ele)?);
        }
    };

    let unsafe_cast = if is_option {
        quote! {}
    } else {
        quote! {
            // Safe since we error out if we can't initialize an entry
            // and Option is repr transparent
            let output = unsafe { core::mem::transmute::<[std::mem::MaybeUninit<#ty>; #len], [#ty; #len]>(output) };
        }
    };

    Ok(quote! {
        #header
        #len_check

        for i in 0..arr.len() {
            let ele = &arr[i];
            #parser
        }

        #unsafe_cast
        output
    })
}

fn gen_field_writer(field: &Field) -> Result<TokenStream, Error> {
    let celeste_rs = celeste_rs();
    if let Some(func) = &field.writing_fn {
        let ident = &field.rust_name;
        let name = &field.yaml_name;
        return Ok(quote! {
            output.insert(
                #celeste_rs::utils::yaml::saphyr::Yaml::String(#name.to_owned()),
                #func(&self.#ident)?
            );
        });
    }

    gen_type_writer(&field.yaml_name, &field.rust_name, &field.rust_type)
}

fn gen_type_writer(name: &str, ident: &Ident, ty: &Type) -> Result<TokenStream, Error> {
    let celeste_rs = celeste_rs();
    match &ty {
        Type::Array(_) => Ok(quote! {
            output.insert(
                #celeste_rs::utils::yaml::saphyr::Yaml::String(#name.to_owned()),
                #celeste_rs::utils::yaml::saphyr::Yaml::Array(self.#ident.iter().map(#celeste_rs::utils::yaml::YamlFile::to_yaml).collect::<Result<Vec<_>,_>>()?)
            );
        }),
        Type::Paren(type_paren) => gen_type_writer(name, ident, &type_paren.elem),
        Type::Path(_) => gen_path_writer(name, ident, ty),
        t => Err(Error::new(
            t.span(),
            format!("Field of type {t:?} is not allowed in YamlFile derive"),
        )),
    }
}

fn gen_path_writer(name: &str, ident: &Ident, ty: &Type) -> Result<TokenStream, Error> {
    let celeste_rs = celeste_rs();
    Ok(if let Some(_ty) = get_option_ty(ty) {
        quote! {
            if let Some(yaml) = self.#ident.to_yaml() {
                output.insert(
                    #celeste_rs::utils::yaml::saphyr::Yaml::String(#name.to_owned()),
                    yaml
                );
            }
        }
    } else {
        quote! {
            output.insert(
                #celeste_rs::utils::yaml::saphyr::Yaml::String(#name.to_owned()),
                self.#ident.to_yaml()?
            );
        }
    })
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.segments.first().is_some_and(|p| p.ident == "Option")
    } else {
        false
    }
}

fn get_option_ty(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        let segment = path.segments.first()?;

        if let PathArguments::AngleBracketed(args) = &segment.arguments {
            if let GenericArgument::Type(t) = &args.args.first()? {
                Some(t)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

struct Field {
    rust_name: Ident,
    yaml_name: String,
    rust_type: Type,
    parsing_fn: Option<Ident>,
    writing_fn: Option<Ident>,
}
