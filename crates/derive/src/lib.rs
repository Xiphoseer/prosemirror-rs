use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::ParseStream, parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident, LitBool,
    Token,
};

struct KeyValue {
    key: syn::Ident,
    value: syn::Ident,
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

struct Marker(syn::Ident);
impl syn::parse::Parse for Marker {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let key = content.parse()?;
        Ok(Marker(key))
    }
}

enum NodeAttribute {
    Inline,
    Defining,
    Group(syn::Ident),
}

#[derive(Clone)]
enum NodeDataKind {
    Fields(syn::Type),
    Unit,
}

#[allow(dead_code)]
struct Node {
    data_kind: NodeDataKind,
    ident: syn::Ident,
    inline: bool,
    defining: bool,
    group: Option<syn::Ident>,
}

struct Schema {
    schema_ty: Ident,
    #[allow(dead_code)]
    node_ty: Ident,
    node_type_ty: Ident,
    nodes: Vec<Node>,
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

#[proc_macro_derive(Node, attributes(prosemirror))]
pub fn my_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let attrs: Vec<KeyValue> = input
        .attrs
        .into_iter()
        .filter_map(|attr| {
            if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "prosemirror" {
                Some(syn::parse2(attr.tokens).expect("Expected ~#[prosemirror(key = ...)]"))
            } else {
                None
            }
        })
        .collect();

    let mut schema = None;
    for attr in attrs {
        if attr.key == "schema" {
            schema = Some(attr.value);
        }
    }

    let schema_name = schema.expect("Expected #[prosemirror(schema = SchemaType)] attribute");

    let edata: DataEnum = if let Data::Enum(edata) = input.data {
        edata
    } else {
        panic!();
    };

    let nodes = edata
        .variants
        .iter()
        .map(|var| {
            let mut inline = None;
            let mut defining = None;
            let mut group = None;
            for attr in &var.attrs {
                if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "prosemirror" {
                    match syn::parse2(attr.tokens.clone())
                        .expect("Expected -#[prosemirror(key = ...)]")
                    {
                        NodeAttribute::Inline => {
                            inline = Some(true);
                        }
                        NodeAttribute::Defining => {
                            defining = Some(true);
                        }
                        NodeAttribute::Group(grp) => {
                            group = Some(grp);
                        }
                    }
                }
            }
            let data_kind = match &var.fields {
                Fields::Named(_fields) => {
                    panic!()
                }
                Fields::Unnamed(fields) => {
                    assert_eq!(fields.unnamed.len(), 1);
                    let first = fields.unnamed.iter().next().unwrap();
                    NodeDataKind::Fields(first.ty.clone())
                }
                Fields::Unit => NodeDataKind::Unit,
            };
            let inline = inline.unwrap_or(false);
            let defining = defining.unwrap_or(false);
            let ident = var.ident.clone();
            Node {
                data_kind,
                ident,
                inline,
                defining,
                group,
            }
        })
        .collect();

    let node_type_ty = format_ident!("{}Type", name);
    let node_ty = name.clone();
    let schema_ty = schema_name.clone();
    let schema = Schema {
        schema_ty,
        node_ty,
        node_type_ty,
        nodes,
    };

    let clone_impl = make_clone(name.clone(), edata);
    let is_block_impl = make_is_block(&schema);
    let type_impl = make_type(&schema);
    let copy_impl = make_copy(&schema);
    let content_impl = make_content(&schema);

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #clone_impl

        impl Node<#schema_name> for #name {
            fn text_node(&self) -> Option<&TextNode<#schema_name>> {
                if let Self::Text(node) = self {
                    Some(node)
                } else {
                    None
                }
            }

            fn new_text_node(node: TextNode<#schema_name>) -> Self {
                Self::Text(node)
            }

            #is_block_impl
            #type_impl

            fn text<A: Into<String>>(text: A) -> Self {
                Self::Text(TextNode {
                    text: Text::from(text.into()),
                    marks: MarkSet::<#schema_name>::default(),
                })
            }

            #content_impl

            fn marks(&self) -> Option<&MarkSet<#schema_name>> {
                None
            }

            fn mark(&self, set: MarkSet<#schema_name>) -> Self {
                // TODO: marks on other nodes
                if let Some(text_node) = self.text_node() {
                    Self::Text(TextNode {
                        marks: set,
                        text: text_node.text.clone(),
                    })
                } else {
                    self.clone()
                }
            }

            #copy_impl
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

fn make_is_block(schema: &Schema) -> TokenStream {
    let var_iter = schema.nodes.iter().map(|var| {
        let name = var.ident.clone();
        let span = name.span();
        let is_block = LitBool::new(!var.inline, span);
        let pat = match var.data_kind {
            NodeDataKind::Unit => quote_spanned!(span=>Self::#name),
            NodeDataKind::Fields(_) => {
                quote_spanned!(span=>Self::#name { .. })
            }
        };
        quote_spanned!(span=> #pat => #is_block)
    });
    quote! {
        fn is_block(&self) -> bool {
            match self {
                #(#var_iter),*
            }
        }
    }
}

fn make_type(schema: &Schema) -> TokenStream {
    let node_type = schema.node_type_ty.clone();
    let var_iter = schema.nodes.iter().map(|var| {
        let name = var.ident.clone();
        let span = name.span();

        let pat = match var.data_kind {
            NodeDataKind::Unit => quote_spanned!(span=>Self::#name),
            NodeDataKind::Fields(_) => {
                quote_spanned!(span=>Self::#name { .. })
            }
        };
        quote_spanned!(span=> #pat => #node_type::#name)
    });
    quote! {
        fn r#type(&self) -> #node_type {
            match self {
                #(#var_iter),*
            }
        }
    }
}

fn make_copy(schema: &Schema) -> TokenStream {
    let schema_ty = schema.schema_ty.clone();
    let var_iter = schema.nodes.iter().map(|var| {
        let name = var.ident.clone();
        let span = name.span();

        match var.data_kind {
            NodeDataKind::Unit => quote_spanned!(span=>Self::#name => Self::#name),
            NodeDataKind::Fields(_) => {
                quote_spanned!(span=>Self::#name(data) => Self::#name(data.copy(map)))
            }
        }
    });
    quote! {
        fn copy<F>(&self, map: F) -> Self
        where
            F: FnOnce(&Fragment<#schema_ty>) -> Fragment<#schema_ty>,
        {
            match self {
                #(#var_iter),*
            }
        }
    }
}

fn make_content(schema: &Schema) -> TokenStream {
    let schema_ty = schema.schema_ty.clone();
    let var_iter = schema.nodes.iter().map(|var| {
        let name = var.ident.clone();
        let span = name.span();

        match &var.data_kind {
            NodeDataKind::Unit => quote_spanned!(span=>Self::#name => None),
            NodeDataKind::Fields(data_ty) => {
                quote_spanned!(span=>Self::#name(data) => <#data_ty as ::prosemirror_model::NodeImpl<#schema_ty>>::content(data))
            }
        }
    });
    quote! {
        fn content(&self) -> Option<&Fragment<#schema_ty>> {
            match self {
                #(#var_iter),*
            }
        }
    }
}

fn make_clone(name: Ident, edata: DataEnum) -> TokenStream {
    let iter = edata.variants.into_iter().map(|var| {
        let name = var.ident;
        let fields = var.fields;

        match fields {
            Fields::Named(nfields) => {
                let names: Vec<_> = nfields.named.iter().map(|f| &f.ident).cloned().collect();
                let tys = nfields.named.iter().map(|f| f.ty.clone());
                quote! {
                    Self::#name{ #(#names),* } => Self::#name{ #(<#tys as Clone>::clone(#names)),* }
                }
            }
            Fields::Unnamed(ufields) => {
                let mut a = vec![];
                let mut b = vec![];
                for (i, field) in ufields.unnamed.into_iter().enumerate() {
                    let name = format_ident!("f{}", i);
                    let ty = field.ty;
                    let clone_expr = quote!(<#ty as Clone>::clone(#name));
                    a.push(name);
                    b.push(clone_expr);
                }
                quote! {
                    Self::#name(#(#a),*) => Self::#name(#(#b),*)
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#name => Self::#name
                }
            }
        }
    });
    let expanded = quote! {
        impl Clone for #name {
            fn clone(&self) -> Self {
                match self {
                    #(#iter),*
                }
            }
        }
    };
    expanded
}
