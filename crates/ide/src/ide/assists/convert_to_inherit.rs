//! Convert `key = key;` into `inherit key;` in non-rec attrset.
use super::{AssistKind, AssistsCtx};
use crate::TextEdit;
use syntax::ast::{self, AstNode};
use syntax::semantic::AttrKind;

pub(super) fn convert_to_inherit(ctx: &mut AssistsCtx<'_>) -> Option<()> {
    let binding = ctx.covering_node::<ast::AttrpathValue>()?;

    // Must be in non-rec attrset.
    let set = ast::AttrSet::cast(binding.syntax().parent()?)?;
    if set.rec_token().is_some() || set.let_token().is_some() {
        return None;
    }

    // RHS should be a single identifier.
    let rhs = match binding.value()?.flatten_paren()? {
        ast::Expr::Ref(rhs) => rhs,
        _ => return None,
    };

    // LHS should be a single static name.
    let mut attrs = binding.attrpath()?.attrs();
    let attr = attrs.next()?;
    if attrs.next().is_some() {
        return None;
    }
    let key = match AttrKind::of(attr) {
        AttrKind::Static(Some(key)) => key,
        _ => return None,
    };

    // LHS should match RHS.
    if key != rhs.token()?.text() {
        return None;
    }

    // Since RHS is already a valid identifier. Not escaping is required.
    ctx.add(
        "convert_to_inherit",
        format!("Convert to `inherit {key}`"),
        AssistKind::RefactorRewrite,
        vec![TextEdit {
            delete: binding.syntax().text_range(),
            insert: format!("inherit {key};").into(),
        }],
    );

    Some(())
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    define_check_assist!(super::convert_to_inherit);

    #[test]
    fn simple() {
        check("{ $0foo = foo; }", expect!["{ inherit foo; }"]);
        check("{ f$0oo = foo; }", expect!["{ inherit foo; }"]);
        check("{ foo $0= foo; }", expect!["{ inherit foo; }"]);
        check("{ foo = f$0oo; }", expect!["{ inherit foo; }"]);
        check("{ fo$0o = fo$1o; }", expect!["{ inherit foo; }"]);

        check_no("$0{ foo = foo; }");
    }

    #[test]
    fn nested() {
        check(
            r#"{ ${("foo")} = (($0foo)); }"#,
            expect!["{ inherit foo; }"],
        );
    }

    #[test]
    fn simple_no() {
        check_no("{ foo $0= bar; }");
        check_no("{ foo.foo $0= foo; }");
    }

    #[test]
    fn rec_attrset() {
        check_no("rec { foo $0= foo; }");
        check_no("let { foo $0= foo; }");
        check_no("let foo $0= foo; in foo");
    }
}