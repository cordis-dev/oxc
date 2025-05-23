use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{AstNode, ast_util::get_declaration_of_variable, context::LintContext, rule::Rule};

fn no_typeof_undefined_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Compare with `undefined` directly instead of using `typeof`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoTypeofUndefined {
    check_global_variables: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `typeof` comparisons with `undefined`.
    ///
    /// ### Why is this bad?
    ///
    /// Checking if a value is `undefined` by using `typeof value === 'undefined'` is needlessly verbose. It's generally better to compare against `undefined` directly. The only time `typeof` is needed is when a global variable potentially does not exists, in which case, using `globalThis.value === undefined` may be better.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// typeof foo === 'undefined';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo === undefined;
    /// ```
    NoTypeofUndefined,
    unicorn,
    pedantic,
    pending
);

impl Rule for NoTypeofUndefined {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(bin_expr) = node.kind() else {
            return;
        };

        if !matches!(
            bin_expr.operator,
            BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::Equality
                | BinaryOperator::Inequality,
        ) {
            return;
        }

        let Expression::UnaryExpression(unary_expr) = &bin_expr.left else {
            return;
        };

        if unary_expr.operator != UnaryOperator::Typeof {
            return;
        }

        if !self.check_global_variables && is_global_variable(&unary_expr.argument, ctx) {
            return;
        }

        if !bin_expr.right.is_specific_string_literal("undefined") {
            return;
        }

        ctx.diagnostic(no_typeof_undefined_diagnostic(bin_expr.span));
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let check_global_variables =
            value.get("checkGlobalVariables").and_then(serde_json::Value::as_bool).unwrap_or(false);

        Self { check_global_variables }
    }
}

fn is_global_variable<'a>(ident: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let Expression::Identifier(ident) = ident else {
        return false;
    };

    get_declaration_of_variable(ident, ctx).is_none()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"typeof a.b", None),
        (r#"typeof a.b > "undefined""#, None),
        (r#"a.b === "undefined""#, None),
        (r#"void a.b === "undefined""#, None),
        (r#"+a.b === "undefined""#, None),
        (r#"++a.b === "undefined""#, None),
        (r#"a.b++ === "undefined""#, None),
        (r"foo === undefined", None),
        (r#"typeof a.b === "string""#, None),
        (r#"typeof foo === "undefined""#, None),
        (r#"foo = 2; typeof foo === "undefined""#, None),
        (r#"/* globals foo: readonly */ typeof foo === "undefined""#, None),
        (r#"/* globals globalThis: readonly */ typeof globalThis === "undefined""#, None),
        (r#""undefined" === typeof a.b"#, None),
        (r#"const UNDEFINED = "undefined"; typeof a.b === UNDEFINED"#, None),
        (r"typeof a.b === `undefined`", None),
    ];

    let fail = vec![
        (r#"typeof a.b === "undefined""#, None),
        (r#"typeof a.b !== "undefined""#, None),
        (r#"typeof a.b == "undefined""#, None),
        (r#"typeof a.b != "undefined""#, None),
        (r"typeof a.b == 'undefined'", None),
        (r#"let foo; typeof foo === "undefined""#, None),
        (r#"const foo = 1; typeof foo === "undefined""#, None),
        (r#"var foo; typeof foo === "undefined""#, None),
        (r#"var foo; var foo; typeof foo === "undefined""#, None),
        (r#"for (const foo of bar) typeof foo === "undefined";"#, None),
        (r#"function foo() {typeof foo === "undefined"}"#, None),
        (r#"function foo(bar) {typeof bar === "undefined"}"#, None),
        (r#"function foo({bar}) {typeof bar === "undefined"}"#, None),
        (r#"function foo([bar]) {typeof bar === "undefined"}"#, None),
        (r#"typeof foo.bar === "undefined""#, None),
        (
            r#"let foo; typeof foo === "undefined""#,
            Some(serde_json::json!({ "checkGlobalVariables": false })),
        ),
        (
            r#"typeof foo === "undefined""#,
            Some(serde_json::json!({ "checkGlobalVariables": true })),
        ),
    ];

    Tester::new(NoTypeofUndefined::NAME, NoTypeofUndefined::PLUGIN, pass, fail).test_and_snapshot();
}
