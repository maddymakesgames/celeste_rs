mod entity;
mod map_element;
mod trigger;

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    token::{Bracket, Colon, Crate, Paren, PathSep, Pound, Pub},
    Attribute,
    DeriveInput,
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

pub(crate) fn celeste_rs() -> TokenStream {
    match crate_name("celeste_rs").expect("Using celeste_rs_macros without celeste_rs :(((((") {
        FoundCrate::Itself => Crate(Span::call_site()).to_token_stream(),
        FoundCrate::Name(named) => Ident::new(&named, Span::call_site()).to_token_stream(),
    }
}

// TODO: probably combine map_element, entity, and trigger into a function with some options or smth
// they're all so similar it feels bad to have so much copied code
// at least entity and trigger can probably work.

#[proc_macro_attribute]
/// Makes a struct usable as the root of a celeste-related save file.
///
/// Specifically, makes a second version of the struct called `Root[struct name]`
/// which has added fields for the xml metadata needed for saves to be loaded.
/// Also creates `From` impl to and from the original struct.
///
/// You likely don't need to use this, it is used in `celeste_rs` to allow `modsavedata` files for
/// Aurora's Additions and Collab Utils 2 to store `SavedSession`s that can still be read by the game.<br>
/// Its only really needed if you're reimplementing `celeste_rs`'s `SaveData` struct or have a `modsavedata`
/// storing xml data.
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

#[proc_macro_derive(MapElement, attributes(child, name, dyn_child, rle))]
/// Derives the `MapElement` trait.
///
/// Every field in the struct needs to be annotated with either `child`, `name`, or `dyn_child`.
/// The struct itself also needs to be annotated with `name`
///
/// #### name
/// The name annotation is used to indicate the element's name in the binary file along with the name of any attributes.<br>
/// For example: a struct representing an element called `box` would look like this
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {}
///
/// ```
///
/// If `Box` has the integer attributes `width` and `specialColorNumber` it would look like this:
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {
///     #[name = "width"]
///     width: Integer,
///     #[name = "specialColorNumber"]
///     special_color_number: Integer,
/// }
/// ```
///
/// This can parse an argument into anything that implements `TryFrom<&EncodedVar>`.
///
/// Specifically with `String` fields, you can also use the `rle` attribute to denote that the field
/// should be read and written using [Run Length Encoding](https://en.wikipedia.org/wiki/Run-length_encoding)
///
/// #### child
/// The `child` annotation is used to indicate that a field is a child element.<br>
/// If this is used, there cannot be a field annotated with `dyn_child`.
///
/// For example: a struct representing a `box` element that has a `position` element as a child would look like this:
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {
///     #[child]
///     pos: Position
/// }
///
/// #[derive(MapElement)]
///#[name = "position"]
/// pub struct Position {
///     #[name = "x"]
///     x: Integer,
///     #[name = "y"]
///     y: Integer,
/// }
/// ```
///
/// #### dyn_child
/// The `dyn_child` annotation is used to indicate that a field should be parsed as a heterogenous array of all child elements.<br>
/// If this is used, there can be no fields annotated with `child`.
///
/// This can be used like this:
/// ```ignore
/// #[derive(MapElement)]
/// pub struct Collection {
///     #[dyn_child]
///     children: Vec<DynMapElement>,
/// }
/// ```
pub fn map_element_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if matches!(input.data, syn::Data::Enum(_) | syn::Data::Union(_)) {
        Error::new(
            input.ident.span(),
            "MapElement can currently only be implemented for structs",
        )
        .into_compile_error()
    } else {
        map_element::map_element_derive(input).unwrap_or_else(Error::into_compile_error)
    }
    .into()
}

#[proc_macro_derive(Entity, attributes(node, name))]
/// Derives the `Entity` trait.
///
/// Every field in the struct needs to be annotated with either `node` or `name`.
/// The struct itself also needs to be annotated with `name`
///
/// #### name
/// The name annotation is used to indicate the entity's name in the binary file along with the name of any attributes.<br>
/// For example: a struct representing an element called `box` would look like this
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {}
///
/// ```
///
/// If `Box` has the integer attributes `width` and `specialColorNumber` it would look like this:
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {
///     #[name = "width"]
///     width: Integer,
///     #[name = "specialColorNumber"]
///     special_color_number: Integer,
/// }
/// ```
///
/// This can parse an argument into anything that implements `TryFrom<&EncodedVar>`.
///
/// #### node
/// The `node` annotation is used to indicate that a field is a child and is `Node`, `Option<Node>`, or `Vec<Node>`.<br>
/// There can only be one field marked with `node` in a struct.
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {
///     #[node]
///     pos: Node
/// }
/// ```
pub fn entity_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if matches!(input.data, syn::Data::Enum(_) | syn::Data::Union(_)) {
        Error::new(
            input.ident.span(),
            "Entity can currently only be implemented for structs",
        )
        .into_compile_error()
    } else {
        entity::entity_derive(input).unwrap_or_else(Error::into_compile_error)
    }
    .into()
}

#[proc_macro_derive(Trigger, attributes(node, name))]
/// Derives the `Trigger` trait.
///
/// Every field in the struct needs to be annotated with either `node` or `name`.
/// The struct itself also needs to be annotated with `name`
///
/// #### name
/// The name annotation is used to indicate the trigger's name in the binary file along with the name of any attributes.<br>
/// For example: a struct representing an element called `box` would look like this
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {}
///
/// ```
///
/// If `Box` has the integer attributes `width` and `specialColorNumber` it would look like this:
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {
///     #[name = "width"]
///     width: Integer,
///     #[name = "specialColorNumber"]
///     special_color_number: Integer,
/// }
/// ```
///
/// This can parse an argument into anything that implements `TryFrom<&EncodedVar>`.
///
/// #### node
/// The `node` annotation is used to indicate that a field is a child and is `Node`, `Option<Node>`, or `Vec<Node>`.<br>
/// There can only be one field marked with `node` in a struct.
/// ```ignore
/// #[derive(MapElement)]
/// #[name = "box"]
/// pub struct Box {
///     #[node]
///     pos: Node
/// }
/// ```
pub fn trigger_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if matches!(input.data, syn::Data::Enum(_) | syn::Data::Union(_)) {
        Error::new(
            input.ident.span(),
            "Trigger can currently only be implemented for structs",
        )
        .into_compile_error()
    } else {
        trigger::trigger_derive(input).unwrap_or_else(Error::into_compile_error)
    }
    .into()
}
