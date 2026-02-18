mod expand;
mod visitor;

use expand::{extract_result_types, transform_tail_err};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    GenericArgument, PathArguments, ReturnType, Stmt, Type, TypePath, parse_macro_input,
    visit_mut::VisitMut,
};
use visitor::SpannedVisitor;

#[proc_macro_attribute]
pub fn spanned_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as syn::ItemFn);

    // Extract Result<T, E> from return type
    let Some((_ok_ty, err_ty)) = extract_result_types(&func.sig.output) else {
        return syn::Error::new_spanned(
            &func.sig.output,
            "#[spanned_error] requires return type Result<T, E>",
        )
        .to_compile_error()
        .into();
    };
    let err_ty = err_ty.clone();

    // Rewrite return type to Result<T, ::spanned_error::SpannedError<E>>
    if let ReturnType::Type(_, ref mut ty) = func.sig.output {
        if let Type::Path(TypePath { path, .. }) = ty.as_mut() {
            if let Some(last) = path.segments.last_mut() {
                if let PathArguments::AngleBracketed(args) = &mut last.arguments {
                    if let Some(GenericArgument::Type(et)) = args.args.last_mut() {
                        *et = syn::parse2(quote! { ::spanned_error::SpannedError<#err_ty> })
                            .expect("spanned_error: failed to parse SpannedError<E> type");
                    }
                }
            }
        }
    }

    // Walk AST and transform ? and return Err(...)
    let mut visitor = SpannedVisitor;
    visitor.visit_block_mut(&mut func.block);

    // Transform tail Err(e)
    transform_tail_err(&mut func.block.stmts);

    // Prepend `use ::spanned_error::__private::kind::*;`
    let use_stmt: Stmt = syn::parse2(quote! { use ::spanned_error::__private::kind::*; })
        .expect("spanned_error: failed to parse use statement");
    func.block.stmts.insert(0, use_stmt);

    quote! { #func }.into()
}
