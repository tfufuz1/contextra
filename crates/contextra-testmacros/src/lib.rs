//! Proc-Macros für Contextra-Tests und Benchmarks (`ctx_test`, `ctx_bench`).
//!
//! Ring: Tooling (dev)
//! Erlaubte Abhängigkeiten: keine Workspace-Abhängigkeiten (AR-1, AR-14)
//! SPEC-Verweis: SPEC §5.1, §5.1a, §5.2, §15.1, L-6, D-91, D-97, D-100

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, ExprLit, ItemFn, Lit, Meta, MetaNameValue, Token};

struct MacroArgs {
    id: String,
    span: proc_macro2::Span,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut id_entry = None;

        while !input.is_empty() {
            if input.peek(syn::Ident) {
                let meta = input.parse::<Meta>()?;
                match meta {
                    Meta::NameValue(MetaNameValue {
                        ref path,
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(ref lit_str),
                                ..
                            }),
                        ..
                    }) if path.is_ident("id") => {
                        if id_entry.is_some() {
                            return Err(syn::Error::new_spanned(
                                &meta,
                                "duplicate argument `id` in attribute",
                            ));
                        }
                        id_entry = Some((lit_str.value(), lit_str.span()));
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            meta,
                            "expected `id = \"...\"` as attribute argument",
                        ));
                    }
                }
            } else {
                return Err(input.error("expected `id = \"...\"`"));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        let (id, span) = id_entry.ok_or_else(|| {
            syn::Error::new(input.span(), "missing mandatory argument `id = \"...\"`")
        })?;

        Ok(Self { id, span })
    }
}

fn validate_test_id(id: &str, span: proc_macro2::Span) -> syn::Result<()> {
    if !id.starts_with("T-") {
        return Err(syn::Error::new(
            span,
            format!("invalid test ID `{id}`: must start with prefix `T-` (e.g. `T-TRACE-1`)"),
        ));
    }
    if id.len() <= 2 {
        return Err(syn::Error::new(
            span,
            format!("invalid test ID `{id}`: must have a non-empty name after `T-`"),
        ));
    }
    let rest = &id["T-".len()..];
    for c in rest.chars() {
        if c.is_ascii_lowercase() {
            return Err(syn::Error::new(
                span,
                format!("invalid test ID `{id}`: lowercase characters not allowed"),
            ));
        }
        if !c.is_ascii_uppercase() && !c.is_ascii_digit() && c != '-' && c != '_' {
            return Err(syn::Error::new(
                span,
                format!("invalid character `{c}` in test ID `{id}`: allowed are uppercase alphanumeric, '-', and '_'"),
            ));
        }
    }
    Ok(())
}

fn validate_bench_id(id: &str, span: proc_macro2::Span) -> syn::Result<()> {
    if !id.starts_with("BM-") {
        return Err(syn::Error::new(
            span,
            format!(
                "invalid benchmark ID `{id}`: must start with prefix `BM-` (e.g. `BM-WRITE-1`)"
            ),
        ));
    }
    if id.len() <= 3 {
        return Err(syn::Error::new(
            span,
            format!("invalid benchmark ID `{id}`: must have a non-empty name after `BM-`"),
        ));
    }
    let rest = &id["BM-".len()..];
    for c in rest.chars() {
        if c.is_ascii_lowercase() {
            return Err(syn::Error::new(
                span,
                format!("invalid benchmark ID `{id}`: lowercase characters not allowed"),
            ));
        }
        if !c.is_ascii_uppercase() && !c.is_ascii_digit() && c != '-' && c != '_' {
            return Err(syn::Error::new(
                span,
                format!("invalid character `{c}` in benchmark ID `{id}`: allowed are uppercase alphanumeric, '-', and '_'"),
            ));
        }
    }
    Ok(())
}

/// Attribut-Makro `#[ctx_test(id = "T-...")]`.
///
/// Prüft zur Compile-Zeit das Vorhandensein und das Format der SPEC-Test-ID.
/// Erzeugt standardmäßig `#[test]` oder `#[tokio::test]`, falls die Zielfunktion `async` ist.
#[proc_macro_attribute]
pub fn ctx_test(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let item_fn = parse_macro_input!(input as ItemFn);

    if let Err(err) = validate_test_id(&args.id, args.span) {
        return err.to_compile_error().into();
    }

    let is_async = item_fn.sig.asyncness.is_some();
    let has_test_attr = item_fn.attrs.iter().any(|attr| {
        attr.path().is_ident("test")
            || attr
                .path()
                .segments
                .last()
                .is_some_and(|s| s.ident == "test")
    });

    let test_attribute = if has_test_attr {
        quote! {}
    } else if is_async {
        quote! { #[tokio::test] }
    } else {
        quote! { #[test] }
    };

    let id_str = &args.id;
    let doc_attr = quote! {
        #[doc = concat!("SPEC-ID: `", #id_str, "`")]
    };

    let expanded = quote! {
        #doc_attr
        #test_attribute
        #item_fn
    };

    TokenStream::from(expanded)
}

/// Attribut-Makro `#[ctx_bench(id = "BM-...")]`.
///
/// Prüft zur Compile-Zeit das Vorhandensein und das Format der SPEC-Benchmark-ID.
#[proc_macro_attribute]
pub fn ctx_bench(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let item_fn = parse_macro_input!(input as ItemFn);

    if let Err(err) = validate_bench_id(&args.id, args.span) {
        return err.to_compile_error().into();
    }

    let id_str = &args.id;
    let doc_attr = quote! {
        #[doc = concat!("SPEC-BENCH-ID: `", #id_str, "`")]
    };

    let expanded = quote! {
        #doc_attr
        #item_fn
    };

    TokenStream::from(expanded)
}
