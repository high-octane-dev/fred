#![feature(abi_thiscall)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, parse_quote};
use syn::{punctuated::Punctuated, token::Comma, BareFnArg, FnArg};

// copied from https://github.com/ultimate-research/skyline-rs/blob/master/skyline_macro/src/lib.rs
fn into_bare_args(args: &Punctuated<FnArg, Comma>) -> Punctuated<BareFnArg, Comma> {
    args.iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                BareFnArg {
                    attrs: pat_type.attrs.clone(),
                    name: None,
                    ty: (*pat_type.ty).clone(),
                }
            } else {
                todo!()
            }
        })
        .collect()
}

fn get_arg_pats(args: &Punctuated<FnArg, Comma>) -> Punctuated<syn::Pat, Comma> {
    args.iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                (*pat_type.pat).clone()
            } else {
                todo!()
            }
        })
        .collect()
}

#[proc_macro_attribute]
pub fn from_offset(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut fn_sig = parse_macro_input!(input as syn::ForeignItemFn);
    let offset = parse_macro_input!(attr as syn::Expr);

    let mut inner_fn_type: syn::TypeBareFn = parse_quote!(extern "C" fn());

    inner_fn_type.abi = fn_sig.sig.abi.clone();

    inner_fn_type.output = fn_sig.sig.output.clone();
    inner_fn_type.variadic = fn_sig.sig.variadic.clone();
    inner_fn_type.inputs = into_bare_args(&fn_sig.sig.inputs);

    let visibility = fn_sig.vis;
    fn_sig.sig.unsafety = Some(syn::token::Unsafe {
        span: Span::call_site(),
    });

    let sig = fn_sig.sig;
    let args = get_arg_pats(&sig.inputs);

    // Generate a shim for the function at the offset
    quote!(
        #visibility #sig {
            let inner = core::mem::transmute::<_,#inner_fn_type>(#offset as usize);
            inner(
                #args
            )
        }
    )
    .into()
}
