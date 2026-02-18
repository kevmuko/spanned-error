use quote::quote;
use syn::{
    Expr, ExprReturn, ExprTry, ItemFn,
    visit_mut::{self, VisitMut},
};

use crate::expand::{is_err_call, wrap_expr};

pub struct SpannedVisitor;

impl VisitMut for SpannedVisitor {
    // Skip closure bodies
    fn visit_expr_closure_mut(&mut self, _: &mut syn::ExprClosure) {
        // do not recurse
    }

    // Skip async blocks
    fn visit_expr_async_mut(&mut self, _: &mut syn::ExprAsync) {
        // do not recurse
    }

    // Skip nested fn items
    fn visit_item_fn_mut(&mut self, _: &mut ItemFn) {
        // do not recurse
    }

    // Transform `expr?` -> match with autoref dispatch
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Recurse first so inner expressions are transformed before outer
        visit_mut::visit_expr_mut(self, expr);

        if let Expr::Try(ExprTry { expr: inner, .. }) = expr {
            let inner_expr = inner.as_ref();
            let wrapped = quote! {
                match #inner_expr {
                    ::core::result::Result::Ok(__spanned_val) => __spanned_val,
                    ::core::result::Result::Err(__spanned_err) => {
                        return ::core::result::Result::Err(
                            (&__spanned_err).spanned_kind().into_spanned(__spanned_err)
                        )
                    }
                }
            };
            *expr = syn::parse2(wrapped).expect("spanned: failed to parse try expansion");
        }
    }

    // Transform `return Err(e)` -> return Err(wrap(e))
    fn visit_expr_return_mut(&mut self, ret: &mut ExprReturn) {
        // Recurse first
        visit_mut::visit_expr_return_mut(self, ret);

        let Some(ref mut return_expr) = ret.expr else {
            return;
        };

        if let Expr::Call(call) = return_expr.as_ref()
            && is_err_call(call)
            && let Some(arg) = call.args.first()
        {
            let wrapped = wrap_expr(arg);
            let new_expr: Expr = syn::parse2(quote! {
                ::core::result::Result::Err(#wrapped)
            })
            .expect("spanned: failed to parse return Err expansion");
            **return_expr = new_expr;
        }
    }
}
