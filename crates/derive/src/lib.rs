use proc_macro2::Ident;
use quote::{format_ident, quote, quote_spanned};
use syn::{Attribute, Data, DataEnum, DeriveInput, Fields, Token, braced, custom_keyword, parse::{Parse, ParseStream, Parser}, parse_macro_input, punctuated::Punctuated, token::{self, Break}};

use crate::parse::{KeyValue, Node, NodeAttribute, NodeDataKind, Schema};

mod impls;
mod parse;

mod pm {
    syn::custom_keyword!(schema);
}

struct PMNodeVariant {
    ident: Ident,
}

impl Parse for PMNodeVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PMNodeVariant {
            ident: input.parse()?,
        })
    }
}

struct PMNode {
    enum_token: Token!(enum),
    ident: Ident,
    brace_token: token::Brace,
    variants: Punctuated<PMNodeVariant, Token![,]>
}

impl Parse for PMNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parser = Punctuated::<PMNodeVariant, Token![,]>::parse_terminated;
        Ok(PMNode {
            enum_token: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            variants: parser(&content)?,
        })
    }
}

struct PMSchema {
    attrs: Vec<Attribute>,
    mod_token: Token!(mod),
    ident: Ident,
    brace_token: token::Brace,

    node: PMNode,
}

impl Parse for PMSchema {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(PMSchema {
            attrs: input.call(Attribute::parse_outer)?,
            mod_token: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            node: content.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn schema(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = parse_macro_input!(item as PMSchema);
    
    let mod_attrs = schema.attrs;
    let mod_name = schema.ident;
    let expanded = quote!{
        #(#mod_attrs)*
        pub mod #mod_name {

        }
};
    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
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
                Fields::Named(fields) => {
                    let mut content = None;

                    for field in &fields.named {
                        if let Some(ident) = &field.ident {
                            if ident == "content" {
                                content = Some(ident.clone());
                            }
                        }
                    }

                    NodeDataKind::Inplace {
                        content
                    }
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

    let clone_impl = impls::make_clone(name.clone(), edata);
    let is_block_impl =  impls::make_is_block(&schema);
    let type_impl =  impls::make_type(&schema);
    let copy_impl =  impls::make_copy(&schema);
    let content_impl =  impls::make_content(&schema);

    // Build the output, possibly using quasi-quotation
    let span = schema.node_ty.span();
    let expanded = quote_spanned! {span=>
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

