use syn::{parse::ParseStream, Token, Ident};

pub struct KeyValue {
    pub key: syn::Ident,
    pub value: syn::Ident,
}

impl syn::parse::Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let key = content.parse()?;
        content.parse::<Token![=]>()?;
        let value = content.parse()?;
        Ok(KeyValue { key, value })
    }
}

pub struct Marker(pub syn::Ident);
impl syn::parse::Parse for Marker {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let key = content.parse()?;
        Ok(Marker(key))
    }
}

pub enum NodeAttribute {
    Inline,
    Defining,
    Group(syn::Ident),
}

#[derive(Clone)]
pub enum NodeDataKind {
    Inplace {
        content: Option<syn::Ident>,
    },
    Fields(syn::Type),
    Unit,
}

#[allow(dead_code)]
pub struct Node {
    pub data_kind: NodeDataKind,
    pub ident: syn::Ident,
    pub inline: bool,
    pub defining: bool,
    pub group: Option<syn::Ident>,
}

pub struct Schema {
    pub schema_ty: Ident,
    #[allow(dead_code)]
    pub node_ty: Ident,
    pub node_type_ty: Ident,
    pub nodes: Vec<Node>,
}

impl syn::parse::Parse for NodeAttribute {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        //let block = Ident::new("block", Span::call_site());
        let lookahead = content.lookahead1();
        if lookahead.peek(Ident) {
            let i: syn::Ident = content.parse()?;
            if i == "inline" {
                Ok(NodeAttribute::Inline)
            } else if i == "defining" {
                Ok(NodeAttribute::Defining)
            } else if i == "group" {
                content.parse::<Token![=]>()?;
                let grp: syn::Ident = content.parse()?;
                Ok(NodeAttribute::Group(grp))
            } else {
                Err(lookahead.error())
            }
        } else {
            Err(lookahead.error())
        }
    }
}