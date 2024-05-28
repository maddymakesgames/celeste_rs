use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    token::{Bracket, Colon, Crate, Paren, PathSep, Pound, Pub},
    Attribute,
    Error,
    Field,
    FieldMutability,
    Fields,
    Ident,
    ItemStruct,
    MacroDelimiter,
    Meta,
    MetaList,
    Path,
    PathSegment,
    Type,
    TypePath,
    Visibility,
};

fn celeste_rs() -> TokenStream {
    match crate_name("celeste_rs").expect("Using celeste_rs_macros without celeste_rs :(((((") {
        FoundCrate::Itself => Crate(Span::call_site()).to_token_stream(),
        FoundCrate::Name(named) => Ident::new(&named, Span::call_site()).to_token_stream(),
    }
}


#[proc_macro_attribute]
pub fn root_tag(
    _args: proc_macro::TokenStream,
    element: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(element as ItemStruct);

    let mut new_struct = input.clone();

    new_struct.ident = format_ident!("Root{}", input.ident);

    let fields = match &mut new_struct.fields {
        Fields::Named(named) => named,
        _ =>
            return Error::new(Span::call_site(), "Used on a non-struct element")
                .to_compile_error()
                .into(),
    };


    let mut serde: Punctuated<PathSegment, PathSep> = Punctuated::new();
    serde.push(Ident::new("serde", Span::call_site()).into());
    let mut docs: Punctuated<PathSegment, PathSep> = Punctuated::new();
    docs.push(Ident::new("doc", Span::call_site()).into());

    let input_ident = input.ident.to_string();

    let mut serde_rename = Attribute {
        pound_token: Pound(Span::call_site()),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket(Span::call_site()),
        meta: syn::Meta::List(MetaList {
            path: Path {
                leading_colon: None,
                segments: serde.clone(),
            },
            delimiter: MacroDelimiter::Paren(Paren(Span::call_site())),
            tokens: quote!(rename = #input_ident),
        }),
    };
    let doc_hidden = Attribute {
        pound_token: Pound(Span::call_site()),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket(Span::call_site()),
        meta: syn::Meta::List(MetaList {
            path: Path {
                leading_colon: None,
                segments: docs,
            },
            delimiter: MacroDelimiter::Paren(Paren(Span::call_site())),
            tokens: quote!(hidden),
        }),
    };

    new_struct.attrs.push(serde_rename.clone());
    new_struct.attrs.push(doc_hidden.clone());

    serde_rename.meta = Meta::List(MetaList {
        path: Path {
            leading_colon: None,
            segments: serde.clone(),
        },
        delimiter: MacroDelimiter::Paren(Paren(Span::call_site())),
        tokens: quote!(rename = "@xmlns:xsi"),
    });

    let string = Type::Path(TypePath {
        qself: None,
        path: Ident::new("String", Span::call_site()).into(),
    });

    fields.named.push(Field {
        attrs: vec![serde_rename.clone(), doc_hidden.clone()],
        vis: Visibility::Public(Pub(Span::call_site())),
        mutability: syn::FieldMutability::None,
        ident: Some(Ident::new("xsi_url", Span::call_site())),
        colon_token: Some(Colon(Span::call_site())),
        ty: string.clone(),
    });

    serde_rename.meta = Meta::List(MetaList {
        path: Path {
            leading_colon: None,
            segments: serde,
        },
        delimiter: MacroDelimiter::Paren(Paren(Span::call_site())),
        tokens: quote!(rename = "@xmlns:xsd"),
    });

    fields.named.push(Field {
        attrs: vec![serde_rename, doc_hidden],
        vis: Visibility::Public(Pub(Span::call_site())),
        mutability: FieldMutability::None,
        ident: Some(Ident::new("xsd_url", Span::call_site())),
        colon_token: Some(Colon(Span::call_site())),
        ty: string,
    });

    let input_name = &input.ident;
    let new_name = &new_struct.ident;
    let destructor = input
        .fields
        .clone()
        .into_iter()
        .filter_map(|f| f.ident)
        .collect::<Vec<_>>();

    let celeste_rs = celeste_rs();

    let struct_doc = format!("A version of {input_name} that is able to be used as a root tag");


    quote! {
        #input

        #[doc = #struct_doc]
        #new_struct

        impl From<#new_name> for #input_name {
            fn from(this: #new_name) -> #input_name {
                let #new_name {#(#destructor,)* ..} = this;

                #input_name {
                    #(#destructor),*
                }
            }
        }

        impl From<#input_name> for #new_name {
            fn from(this: #input_name) -> #new_name {
                let #input_name {#(#destructor,)* ..} = this;

                #new_name {
                    #(#destructor,)*
                    xsi_url: #celeste_rs::saves::ops::XSI_URL.to_owned(),
                    xsd_url: #celeste_rs::saves::ops::XSD_URL.to_owned(),
                }
            }
        }
    }
    .into()
}
