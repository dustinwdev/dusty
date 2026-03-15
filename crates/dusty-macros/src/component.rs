use proc_macro2::TokenStream;
use quote::quote;
use syn::parse2;
use syn::spanned::Spanned;
use syn::{FnArg, Ident, ItemFn, Pat, ReturnType, Type};

use crate::error::err;
use crate::prop::{DefaultValue, PropAttr};

/// A parsed component parameter (excluding `cx: Scope`).
struct Param {
    name: Ident,
    ty: Type,
    prop: PropAttr,
}

/// Entry point: parse the function, validate, and generate code.
pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let func: ItemFn = parse2(input)?;
    validate(&func)?;

    let comp_name = &func.sig.ident;
    let params = extract_params(&func)?;
    let body = &func.block;

    let struct_def = gen_struct(comp_name, &params);
    let builder_impl = gen_builder(comp_name, &params);
    let view_impl = gen_view_impl(comp_name, &params, body);

    Ok(quote! {
        #struct_def
        #builder_impl
        #view_impl
    })
}

/// Validate that the function has the expected shape:
/// - First param is `cx: Scope`
/// - Return type is `-> Node`
fn validate(func: &ItemFn) -> syn::Result<()> {
    if func.sig.inputs.is_empty() {
        return Err(err(
            func.sig.ident.span(),
            "#[component] function must have `cx: Scope` as its first parameter",
        ));
    }

    let first = &func.sig.inputs[0];
    match first {
        FnArg::Typed(pat_type) => {
            let ty_str = type_to_string(&pat_type.ty);
            if ty_str != "Scope" {
                return Err(err(
                    pat_type.ty.span(),
                    "#[component] function's first parameter must be `cx: Scope`",
                ));
            }
        }
        FnArg::Receiver(_) => {
            return Err(err(
                first.span(),
                "#[component] function must have `cx: Scope` as its first parameter, not `self`",
            ));
        }
    }

    match &func.sig.output {
        ReturnType::Default => {
            return Err(err(
                func.sig.ident.span(),
                "#[component] function must return `-> Node`",
            ));
        }
        ReturnType::Type(_, ty) => {
            let ty_str = type_to_string(ty);
            if ty_str != "Node" {
                return Err(err(
                    ty.span(),
                    "#[component] function must return `-> Node`",
                ));
            }
        }
    }

    Ok(())
}

/// Extract component parameters (all params after `cx: Scope`).
fn extract_params(func: &ItemFn) -> syn::Result<Vec<Param>> {
    func.sig
        .inputs
        .iter()
        .skip(1) // skip cx: Scope
        .map(|arg| {
            let FnArg::Typed(pat_type) = arg else {
                return Err(err(arg.span(), "unexpected `self` parameter"));
            };

            let Pat::Ident(pat_ident) = &*pat_type.pat else {
                return Err(err(pat_type.pat.span(), "expected a simple identifier"));
            };

            let name = pat_ident.ident.clone();
            let ty = (*pat_type.ty).clone();

            let mut prop = PropAttr::default();
            for attr in &pat_type.attrs {
                if attr.path().is_ident("prop") {
                    prop = attr.parse_args::<PropAttr>()?;
                }
            }

            Ok(Param { name, ty, prop })
        })
        .collect()
}

/// Generate the props struct definition.
fn gen_struct(name: &Ident, params: &[Param]) -> TokenStream {
    let fields: Vec<_> = params
        .iter()
        .map(|p| {
            let field_name = &p.name;
            let ty = &p.ty;
            if p.prop.optional {
                quote! { #field_name: Option<#ty> }
            } else {
                quote! { #field_name: #ty }
            }
        })
        .collect();

    quote! {
        pub struct #name {
            #(#fields,)*
        }
    }
}

/// Generate `new()` constructor and builder methods.
fn gen_builder(name: &Ident, params: &[Param]) -> TokenStream {
    let new_params: Vec<_> = params
        .iter()
        .filter(|p| p.prop.is_required())
        .map(|p| {
            let pname = &p.name;
            let ty = &p.ty;
            if p.prop.into {
                quote! { #pname: impl Into<#ty> }
            } else {
                quote! { #pname: #ty }
            }
        })
        .collect();

    let new_fields: Vec<_> = params
        .iter()
        .map(|p| {
            let pname = &p.name;
            if p.prop.is_required() {
                if p.prop.into {
                    quote! { #pname: #pname.into() }
                } else {
                    quote! { #pname }
                }
            } else if p.prop.optional {
                quote! { #pname: None }
            } else {
                match &p.prop.default {
                    Some(DefaultValue::Expr(expr)) => quote! { #pname: #expr },
                    Some(DefaultValue::Trait) => quote! { #pname: Default::default() },
                    None => unreachable!(),
                }
            }
        })
        .collect();

    let builder_methods: Vec<_> = params
        .iter()
        .filter(|p| !p.prop.is_required())
        .map(|p| {
            let pname = &p.name;
            let ty = &p.ty;
            if p.prop.optional {
                // optional builder always takes impl Into<T>, wraps in Some
                quote! {
                    #[must_use]
                    pub fn #pname(mut self, #pname: impl Into<#ty>) -> Self {
                        self.#pname = Some(#pname.into());
                        self
                    }
                }
            } else if p.prop.into {
                quote! {
                    #[must_use]
                    pub fn #pname(mut self, #pname: impl Into<#ty>) -> Self {
                        self.#pname = #pname.into();
                        self
                    }
                }
            } else {
                quote! {
                    #[must_use]
                    pub fn #pname(mut self, #pname: #ty) -> Self {
                        self.#pname = #pname;
                        self
                    }
                }
            }
        })
        .collect();

    quote! {
        impl #name {
            #[must_use]
            pub fn new(#(#new_params),*) -> Self {
                Self {
                    #(#new_fields,)*
                }
            }

            #(#builder_methods)*
        }
    }
}

/// Generate the `View` impl.
fn gen_view_impl(name: &Ident, params: &[Param], body: &syn::Block) -> TokenStream {
    let field_names: Vec<_> = params.iter().map(|p| &p.name).collect();
    let name_str = name.to_string();
    let destructure = if field_names.is_empty() {
        quote! { let _ = self; }
    } else {
        let fields = &field_names;
        quote! {
            let #name { #(#fields),* } = self;
        }
    };

    quote! {
        impl ::dusty_core::view::View for #name {
            fn build(self, cx: ::dusty_reactive::Scope) -> ::dusty_core::node::Node {
                #destructure
                let __inner = #body;
                ::dusty_core::node::Node::Component(::dusty_core::node::ComponentNode {
                    name: #name_str,
                    child: Box::new(__inner),
                })
            }
        }
    }
}

/// Converts a `Type` to a simple string for comparison.
fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map_or_else(String::new, |s| s.ident.to_string()),
        _ => String::new(),
    }
}
