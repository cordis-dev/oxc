use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct SymbolDescription;

fn symbol_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected Symbol to have a description.").with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require symbol descriptions.
    ///
    /// ### Why is this bad?
    ///
    /// The Symbol function may have an optional description.
    ///
    /// ```js
    /// var foo = Symbol("some description");
    ///
    /// var someString = "some description";
    /// var bar = Symbol(someString);
    /// ```
    ///
    /// Using `description` promotes easier debugging: when a symbol is logged the description is used:
    /// ```js
    /// var foo = Symbol("some description");
    ///
    /// console.log(foo);
    /// // prints - Symbol(some description)
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var foo = Symbol();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var foo = Symbol("some description");
    /// ```
    SymbolDescription,
    eslint,
    pedantic,
);

impl Rule for SymbolDescription {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };

        if ident.name == "Symbol"
            && call_expr.arguments.is_empty()
            && ctx.scoping().root_unresolved_references().contains_key(ident.name.as_str())
        {
            ctx.diagnostic(symbol_description_diagnostic(call_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"Symbol("Foo");"#,
        r#"var foo = "foo"; Symbol(foo);"#,
        "var Symbol = function () {}; Symbol();",
        "Symbol(); var Symbol = function () {};",
        "function bar() { var Symbol = function () {}; Symbol(); }",
        "function bar(Symbol) { Symbol(); }",
    ];

    let fail = vec!["Symbol();", "Symbol(); Symbol = function () {};"];

    Tester::new(SymbolDescription::NAME, SymbolDescription::PLUGIN, pass, fail).test_and_snapshot();
}
