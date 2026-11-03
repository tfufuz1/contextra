//! # Crate `contextra-testmacros`
//!
//! Proc-Makros für die Contextra-Testinfrastruktur:
//! - `#[ctx_test(id = "T-...")]`: Validiert SPEC-Test-IDs und erzeugt Test-Hooks.
//! - `#[ctx_bench(id = "BM-...")]`: Validiert SPEC-Benchmark-IDs und erzeugt Benchmark-Hooks.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, ExprLit, ItemFn, Lit, Meta, MetaNameValue, Token};

struct MacroArgs {
    id: String,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut id = None;

        while !input.is_empty() {
            if input.peek(syn::Ident) {
                let meta = input.parse::<Meta>()?;
                match meta {
                    Meta::NameValue(MetaNameValue {
                        path,
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }),
                        ..
                    }) if path.is_ident("id") => {
                        id = Some(lit_str.value());
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

        let id = id.ok_or_else(|| {
            syn::Error::new(input.span(), "missing mandatory argument `id = \"...\"`")
        })?;

        Ok(Self { id })
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
    for c in id["T-".len()..].chars() {
        if !c.is_ascii_alphanumeric() && c != '-' && c != '_' {
            return Err(syn::Error::new(
                span,
                format!("invalid character `{c}` in test ID `{id}`: allowed are alphanumeric, '-', and '_'"),
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
    for c in id["BM-".len()..].chars() {
        if !c.is_ascii_alphanumeric() && c != '-' && c != '_' {
            return Err(syn::Error::new(
                span,
                format!("invalid character `{c}` in benchmark ID `{id}`: allowed are alphanumeric, '-', and '_'"),
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

    if let Err(err) = validate_test_id(&args.id, proc_macro2::Span::call_site()) {
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

    if let Err(err) = validate_bench_id(&args.id, proc_macro2::Span::call_site()) {
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
