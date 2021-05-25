use quote::{format_ident, quote_spanned};
use proc_macro2::{Ident, TokenStream};
use syn::{DataEnum, Fields, LitBool, spanned::Spanned};

use crate::parse::{NodeDataKind, Schema};

pub fn make_is_block(schema: &Schema) -> TokenStream {
    let var_iter = schema.nodes.iter().map(|var| {
        let name = var.ident.clone();
        let span = name.span();
        let is_block = LitBool::new(!var.inline, span);
        let pat = match var.data_kind {
            NodeDataKind::Unit => quote_spanned!(span=>Self::#name),
            NodeDataKind::Fields(_) | NodeDataKind::Inplace { .. } => {
                quote_spanned!(span=>Self::#name { .. })
            }
        };
        quote_spanned!(span=> #pat => #is_block)
    });
    let span = schema.node_ty.span();
    quote_spanned! {span=>
        fn is_block(&self) -> bool {
            match self {
                #(#var_iter),*
            }
        }
    }
}

pub fn make_type(schema: &Schema) -> TokenStream {
    let node_type = schema.node_type_ty.clone();
    let var_iter = schema.nodes.iter().map(|var| {
        let name = var.ident.clone();
        let span = name.span();

        let pat = match var.data_kind {
            NodeDataKind::Unit => quote_spanned!(span=>Self::#name),
            NodeDataKind::Fields(_) | NodeDataKind::Inplace { .. } => {
                quote_spanned!(span=>Self::#name { .. })
            }
        };
        quote_spanned!(span=> #pat => #node_type::#name)
    });
    let span = schema.node_ty.span();
    quote_spanned! {span=>
        fn r#type(&self) -> #node_type {
            match self {
                #(#var_iter),*
            }
        }
    }
}

pub fn make_copy(schema: &Schema) -> TokenStream {
    let schema_ty = schema.schema_ty.clone();
    let var_iter = schema.nodes.iter().map(|var| {
        let name = var.ident.clone();
        let span = name.span();

        match &var.data_kind {
            NodeDataKind::Unit => quote_spanned!(span=>Self::#name => Self::#name),
            NodeDataKind::Fields(_) => {
                quote_spanned!(span=>Self::#name(data) => Self::#name(data.copy(map)))
            }
            NodeDataKind::Inplace { content } => {
                if let Some(content_field_name) = content {
                    quote_spanned!(span=>Self::#name { #content_field_name } => Self::#name { #content_field_name: map(#content_field_name) })
                } else {
                    quote_spanned!(span=>Self::#name{} => Self::#name{})
                }
            }
        }
    });
    let span = schema.node_ty.span();
    quote_spanned! {span=>
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

pub fn make_content(schema: &Schema) -> TokenStream {
    let schema_ty = schema.schema_ty.clone();
    let var_iter = schema.nodes.iter().map(|var| {
        let name = var.ident.clone();
        let span = name.span();

        match &var.data_kind {
            NodeDataKind::Unit => quote_spanned!(span=>Self::#name => None),
            NodeDataKind::Fields(data_ty) => {
                quote_spanned!(span=>Self::#name(data) => <#data_ty as ::prosemirror_model::NodeImpl<#schema_ty>>::content(data))
            }
            &NodeDataKind::Inplace { ref content } => {
                if let Some(content_field_name) = content {
                    quote_spanned!(span=>Self::#name{ #content_field_name, .. } => Some(#content_field_name))
                } else {
                    quote_spanned!(span=>Self::#name{} => None)
                }
            }
        }
    });
    let span = schema.node_ty.span();
    quote_spanned! {span=>
        fn content(&self) -> Option<&Fragment<#schema_ty>> {
            match self {
                #(#var_iter),*
            }
        }
    }
}

pub fn make_clone(name: Ident, edata: DataEnum) -> TokenStream {
    let iter = edata.variants.into_iter().map(|var| {
        let name = var.ident;
        let fields = var.fields;

        match fields {
            Fields::Named(nfields) => {
                let names: Vec<_> = nfields.named.iter().map(|f: &syn::Field| {
                    f.ident.as_ref().unwrap().clone()
                }).collect();
                let clone_variants = nfields.named.iter().map(|f: &syn::Field| {
                    let ty = f.ty.clone();
                    let span = f.ty.span();
                    let name = f.ident.as_ref().unwrap().clone();
                    quote_spanned! (span=> #name: <#ty as Clone>::clone(#name))
                });
                let span = name.span();
                quote_spanned! {span=>
                    Self::#name{ #(#names),* } => Self::#name{ #(#clone_variants),* }
                }
            }
            Fields::Unnamed(ufields) => {
                let mut a = vec![];
                let mut b = vec![];
                for (i, field) in ufields.unnamed.into_iter().enumerate() {
                    let name = format_ident!("f{}", i);
                    let ty: syn::Type = field.ty;
                    let span = ty.span();
                    let clone_expr = quote_spanned!(span => <#ty as Clone>::clone(#name));
                    a.push(name);
                    b.push(clone_expr);
                }
                let span = name.span();
                quote_spanned! {span=>
                    Self::#name(#(#a),*) => Self::#name(#(#b),*)
                }
            }
            Fields::Unit => {
                let span = name.span();
                quote_spanned! {span=>
                    Self::#name => Self::#name
                }
            }
        }
    });
    let span = name.span();
    let expanded = quote_spanned! {span=>
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
