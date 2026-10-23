//! `eslint/func-style`: convert between function declarations and function expressions.
//!
//! - `expression` style: `function foo(a) {}` becomes `const foo = function (a) {};`, keeping
//!   `async`, generators and type parameters. Also behind `export`, but not `export default`.
//! - `declaration` style: `const foo = function (a) {};` and `const foo = (a) => a;` become
//!   `function foo(a) {}` and `function foo(a) { return a; }`. Arrow functions are converted only when
//!   they do not use `arguments` (the rule already skips arrows that use `this` or `super`), and
//!   function expressions only when they are anonymous or have the same name.
//!
//! A dangerous fix: declarations are hoisted and expressions are not, so calls before the definition
//! change behavior, and `const` bindings cannot be reassigned while function bindings can. Only for
//! a single declaration in a statement list, without TypeScript annotations on the variable, and
//! without comments.

use oxc_ast::{
    AstKind,
    ast::{ArrowFunctionExpression, BindingPattern, Expression, Function},
};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::is_statement_list},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    if fcx.ctx.has_comments_between(span) {
        return vec![];
    }
    let replacement = if fcx.text() == "Expected a function expression." {
        to_expression(fcx, span)
    } else if fcx.text() == "Expected a function declaration." {
        to_declaration(fcx, span)
    } else {
        None
    };
    let Some((replaced, text)) = replacement else { return vec![] };
    vec![
        fcx.fixer(FixKind::DangerousFix)
            .replace(replaced, text)
            .with_message("Change the function style"),
    ]
}

/// `function foo(a) {}` → `const foo = function (a) {};`
fn to_expression(fcx: &FixCtx, span: Span) -> Option<(Span, String)> {
    let node = fcx.exact(span, |node| matches!(node.kind(), AstKind::Function(_)))?;
    let AstKind::Function(function) = node.kind() else { return None };
    let id = function.id.as_ref()?;
    let parent = fcx.nodes().parent_kind(node.id());
    let in_export =
        matches!(parent, AstKind::ExportDeclaration(_) | AstKind::ExportNamedDeclaration(_));
    if function.declare || function.body.is_none() || !(in_export || is_statement_list(&parent)) {
        return None;
    }
    let rest = fcx.source_range(Span::new(id.span.end, function.span.end));
    let prefix = if function.r#async { "async " } else { "" };
    let star = if function.generator { "*" } else { "" };
    Some((
        function.span,
        format!("const {} = {prefix}function{star} {};", id.name, rest.trim_start()),
    ))
}

/// `const foo = function (a) {};` → `function foo(a) {}`
fn to_declaration(fcx: &FixCtx, span: Span) -> Option<(Span, String)> {
    let node = fcx.exact(span, |node| matches!(node.kind(), AstKind::VariableDeclarator(_)))?;
    let AstKind::VariableDeclarator(declarator) = node.kind() else { return None };
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else { return None };
    let declaration_id = fcx.nodes().parent_id(node.id());
    let AstKind::VariableDeclaration(declaration) = fcx.nodes().kind(declaration_id) else {
        return None;
    };
    let parent = fcx.nodes().parent_kind(declaration_id);
    let in_export =
        matches!(parent, AstKind::ExportDeclaration(_) | AstKind::ExportNamedDeclaration(_));
    if declaration.declarations.len() != 1
        || declaration.declare
        || declarator.type_annotation.is_some()
        || !(in_export || is_statement_list(&parent))
    {
        return None;
    }
    let name = ident.name.as_str();
    let text = match declarator.init.as_ref()?.get_inner_expression() {
        Expression::FunctionExpression(function) => function_declaration(fcx, function, name)?,
        Expression::ArrowFunctionExpression(arrow) => arrow_declaration(fcx, arrow, name)?,
        _ => return None,
    };
    Some((declaration.span, text))
}

fn function_declaration(fcx: &FixCtx, function: &Function, name: &str) -> Option<String> {
    if function.id.as_ref().is_some_and(|id| id.name != name) {
        return None;
    }
    let start =
        function.type_parameters.as_ref().map_or(function.params.span.start, |t| t.span.start);
    let rest = fcx.source_range(Span::new(start, function.span.end));
    let prefix = if function.r#async { "async " } else { "" };
    let star = if function.generator { "*" } else { "" };
    Some(format!("{prefix}function{star} {name}{rest}"))
}

fn arrow_declaration(fcx: &FixCtx, arrow: &ArrowFunctionExpression, name: &str) -> Option<String> {
    if fcx.source_range(arrow.span).contains("arguments") {
        return None;
    }
    let type_parameters = arrow.type_parameters.as_ref().map_or("", |t| fcx.source_range(t.span));
    let params = fcx.source_range(arrow.params.span);
    // A single parameter may be written without parentheses.
    let params = if params.starts_with('(') { params.to_string() } else { format!("({params})") };
    let return_type = arrow.return_type.as_ref().map_or("", |t| fcx.source_range(t.span));
    let body = match arrow.get_expression() {
        Some(expression) => format!("{{ return {}; }}", fcx.source_range(expression.span())),
        None => fcx.source_range(arrow.body.span()).to_string(),
    };
    let prefix = if arrow.r#async { "async " } else { "" };
    Some(format!("{prefix}function {name}{type_parameters}{params}{return_type} {body}"))
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{rule::RuleMeta, rules::eslint::func_style::FuncStyle, tester::Tester};

    // The default style is `expression`.
    let pass = vec![("var foo = function() {};", None)];
    let fail = vec![("function foo() {}", None)];

    let declaration = Some(json!(["declaration"]));
    let fix = vec![
        (
            "function foo(a) { return a; }",
            "const foo = function (a) { return a; };",
            None,
            FixKind::DangerousFix,
        ),
        ("function foo(a) { return a; }", "function foo(a) { return a; }", None, FixKind::SafeFix),
        (
            "async function foo() {}",
            "const foo = async function () {};",
            None,
            FixKind::DangerousFix,
        ),
        ("function* foo() {}", "const foo = function* () {};", None, FixKind::DangerousFix),
        (
            "export function foo() {}",
            "export const foo = function () {};",
            None,
            FixKind::DangerousFix,
        ),
        (
            "var foo = function() {};",
            "function foo() {}",
            declaration.clone(),
            FixKind::DangerousFix,
        ),
        (
            "const foo = async function (a) { await a; };",
            "async function foo(a) { await a; }",
            declaration.clone(),
            FixKind::DangerousFix,
        ),
        (
            "const foo = function* () { yield 1; };",
            "function* foo() { yield 1; }",
            declaration.clone(),
            FixKind::DangerousFix,
        ),
        (
            "const foo = (a) => a + 1;",
            "function foo(a) { return a + 1; }",
            declaration.clone(),
            FixKind::DangerousFix,
        ),
        (
            "const foo = a => { return a; };",
            "function foo(a) { return a; }",
            declaration.clone(),
            FixKind::DangerousFix,
        ),
        (
            "export const foo = function () {};",
            "export function foo() {}",
            declaration.clone(),
            FixKind::DangerousFix,
        ),
        // `arguments` means something else in an arrow function.
        (
            "function f() { const foo = () => arguments[0]; }",
            "function f() { const foo = () => arguments[0]; }",
            declaration.clone(),
            FixKind::DangerousFix,
        ),
        // A differently named function expression.
        (
            "var foo = function bar() {};",
            "var foo = function bar() {};",
            declaration,
            FixKind::DangerousFix,
        ),
    ];

    Tester::new(FuncStyle::NAME, FuncStyle::PLUGIN, pass, fail).expect_fix(fix).test();
}
