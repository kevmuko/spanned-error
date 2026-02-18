use quote::quote;
use syn::{Expr, GenericArgument, PathArguments, ReturnType, Stmt, Type, TypePath};

/// Extract the error type `E` from `Result<T, E>`.
/// Checks that the last path segment is `Result` with exactly 2 generic args.
pub fn extract_result_types(ret: &ReturnType) -> Option<(&Type, &Type)> {
    let ReturnType::Type(_, ty) = ret else {
        return None;
    };

    let Type::Path(TypePath { path, .. }) = ty.as_ref() else {
        return None;
    };

    let last = path.segments.last()?;
    if last.ident != "Result" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };

    if args.args.len() != 2 {
        return None;
    }

    let GenericArgument::Type(ok_ty) = &args.args[0] else {
        return None;
    };
    let GenericArgument::Type(err_ty) = &args.args[1] else {
        return None;
    };

    Some((ok_ty, err_ty))
}

/// Build the autoref dispatch expression:
/// ```ignore
/// {
///     let __spanned_err = <expr>;
///     (&__spanned_err).spanned_kind().into_spanned(__spanned_err)
/// }
/// ```
pub fn wrap_expr(expr: &Expr) -> proc_macro2::TokenStream {
    quote! {{
        let __spanned_err = #expr;
        (&__spanned_err).spanned_kind().into_spanned(__spanned_err)
    }}
}

/// Check if a call expression is `Err(...)`.
pub fn is_err_call(call: &syn::ExprCall) -> bool {
    if let Expr::Path(ep) = call.func.as_ref() {
        let last = ep.path.segments.last();
        matches!(last, Some(seg) if seg.ident == "Err")
    } else {
        false
    }
}

/// Transform a tail `Err(e)` expression (last expression without semicolon).
pub fn transform_tail_err(stmts: &mut [Stmt]) {
    let Some(last) = stmts.last_mut() else {
        return;
    };

    let Stmt::Expr(expr, None) = last else {
        return;
    };

    if let Expr::Call(call) = expr
        && is_err_call(call)
        && let Some(arg) = call.args.first()
    {
        let wrapped = wrap_expr(arg);
        let new_expr: Expr = syn::parse2(quote! {
            ::core::result::Result::Err(#wrapped)
        })
        .expect("spanned: failed to parse tail Err expansion");
        *expr = new_expr;
    }
}
