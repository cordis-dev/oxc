//! `eslint/no-undefined`: replace a read of the global `undefined` with `void 0`.
//!
//! Only reads of the unshadowed global are fixed. Declarations (`var undefined`, parameters),
//! writes and `export { undefined }` are reported but not fixed. `void 0` is wrapped in parentheses
//! where a unary expression would bind differently, for example `(void 0).a`. The shorthand
//! property `{ undefined }` becomes `{ undefined: void 0 }`.

use oxc_ast::{AstKind, ast::BinaryExpression};
use oxc_span::GetSpan;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{Binding, binding_of, starts_statement, statement_boundary_before},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::IdentifierReference(_)))
    else {
        return vec![];
    };
    let AstKind::IdentifierReference(ident) = node.kind() else { return vec![] };
    if binding_of(fcx, ident) != Binding::Global
        || fcx.scoping().get_reference(ident.reference_id()).is_write()
    {
        return vec![];
    }

    let parent = fcx.nodes().parent_kind(node.id());
    let replacement = match parent {
        AstKind::ObjectProperty(prop) if prop.shorthand => {
            return vec![
                fcx.fixer(FixKind::SafeFix)
                    .replace(prop.span, "undefined: void 0")
                    .with_message("Replace `undefined` with `void 0`"),
            ];
        }
        // Type positions (`typeof undefined` in a type, `implements undefined`) and module
        // references take a name, not an expression.
        AstKind::ExportSpecifier(_)
        | AstKind::TSTypeQuery(_)
        | AstKind::TSQualifiedName(_)
        | AstKind::TSTypeReference(_)
        | AstKind::TSClassImplements(_)
        | AstKind::TSInterfaceHeritage(_)
        | AstKind::TSImportEqualsDeclaration(_) => return vec![],
        // `extends` takes a left-hand-side expression.
        AstKind::Class(class)
            if class.heritage.as_ref().is_some_and(|h| h.expression.span() == span) =>
        {
            "(void 0)"
        }
        AstKind::StaticMemberExpression(member) if member.object.span() == span => "(void 0)",
        AstKind::ComputedMemberExpression(member) if member.object.span() == span => "(void 0)",
        AstKind::PrivateFieldExpression(member) if member.object.span() == span => "(void 0)",
        AstKind::CallExpression(call) if call.callee.span() == span => "(void 0)",
        AstKind::NewExpression(new) if new.callee.span() == span => "(void 0)",
        AstKind::TaggedTemplateExpression(tagged) if tagged.tag.span() == span => "(void 0)",
        AstKind::BinaryExpression(BinaryExpression {
            operator: BinaryOperator::Exponential,
            left,
            ..
        }) if left.span() == span => "(void 0)",
        // Postfix TypeScript operators: `void 0!.a` would parse as `void (0!.a)`.
        AstKind::TSNonNullExpression(_) | AstKind::TSInstantiationExpression(_) => "(void 0)",
        _ => "void 0",
    };
    // A statement that now starts with `(` could join the previous one.
    if replacement.starts_with('(')
        && starts_statement(fcx, node.id())
        && !statement_boundary_before(fcx, span.start)
    {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::SafeFix)
            .replace(span, replacement)
            .with_message("Replace `undefined` with `void 0`"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_undefined::NoUndefined, tester::Tester};

    let pass = vec!["void 0"];
    let fail = vec!["undefined"];

    let fix = vec![
        ("undefined", "void 0", None, FixKind::SafeFix),
        ("f(undefined)", "f(void 0)", None, FixKind::SafeFix),
        ("a[undefined]", "a[void 0]", None, FixKind::SafeFix),
        ("x = undefined", "x = void 0", None, FixKind::SafeFix),
        ("var x = undefined", "var x = void 0", None, FixKind::SafeFix),
        ("({ bar: undefined })", "({ bar: void 0 })", None, FixKind::SafeFix),
        ("({ undefined })", "({ undefined: void 0 })", None, FixKind::SafeFix),
        ("({ [undefined]: foo })", "({ [void 0]: foo })", None, FixKind::SafeFix),
        ("function f(a = undefined) {}", "function f(a = void 0) {}", None, FixKind::SafeFix),
        ("typeof undefined", "typeof void 0", None, FixKind::SafeFix),
        ("x = undefined.a", "x = (void 0).a", None, FixKind::SafeFix),
        ("x = undefined[0]", "x = (void 0)[0]", None, FixKind::SafeFix),
        ("x = undefined()", "x = (void 0)()", None, FixKind::SafeFix),
        ("x = undefined ** 2", "x = (void 0) ** 2", None, FixKind::SafeFix),
        ("undefined.a", "(void 0).a", None, FixKind::SafeFix),
        ("const f = () => undefined.a", "const f = () => (void 0).a", None, FixKind::SafeFix),
        ("foo();\nundefined.a", "foo();\n(void 0).a", None, FixKind::SafeFix),
        // The statement would start with `(` after a line without a semicolon.
        ("foo()\nundefined.a", "foo()\nundefined.a", None, FixKind::SafeFix),
        ("let a = {}\nundefined.a", "let a = {}\nundefined.a", None, FixKind::SafeFix),
        // A line comment ending in `;` is not a statement end.
        ("foo() // done;\nundefined.a", "foo() // done;\nundefined.a", None, FixKind::SafeFix),
        ("class A extends undefined {}", "class A extends (void 0) {}", None, FixKind::SafeFix),
        ("x = undefined!.a", "x = (void 0)!.a", None, FixKind::SafeFix),
        ("x = undefined as any", "x = void 0 as any", None, FixKind::SafeFix),
        // A type query takes a name, not an expression.
        ("type T = typeof undefined;", "type T = typeof undefined;", None, FixKind::SafeFix),
        // Declarations and writes are not fixed.
        ("var undefined;", "var undefined;", None, FixKind::SafeFix),
        ("function f(undefined) {}", "function f(undefined) {}", None, FixKind::SafeFix),
        ("undefined = true", "undefined = true", None, FixKind::SafeFix),
        (
            "function f() { var undefined; return undefined; }",
            "function f() { var undefined; return undefined; }",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoUndefined::NAME, NoUndefined::PLUGIN, pass, fail).expect_fix(fix).test();
}
