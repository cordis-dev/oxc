//! Helpers shared by fork fixers.
use std::borrow::Cow;

use oxc_ast::ast::{Argument, Expression};
use oxc_span::{GetSpan, Span};

use super::FixCtx;

/// Skips whitespace, comments and, if `skip_close_parens`, closing parentheses starting at `pos`.
/// Returns the offset of the next significant character, or the end of the source.
pub fn skip_trivia(fcx: &FixCtx, mut pos: u32, skip_close_parens: bool) -> u32 {
    let source = fcx.source_text().as_bytes();
    loop {
        let Some(&byte) = source.get(pos as usize) else { return pos };
        if byte.is_ascii_whitespace() || (skip_close_parens && byte == b')') {
            pos += 1;
            continue;
        }
        if let Some(comment) = fcx.ctx.get_comment_at(pos).filter(|c| c.span.start == pos) {
            pos = comment.span.end;
            continue;
        }
        // Non-ASCII whitespace such as U+00A0 or U+FEFF.
        if byte >= 0x80
            && let Some(ch) = fcx.source_text()[pos as usize..].chars().next()
            && ch.is_whitespace()
        {
            pos += u32::try_from(ch.len_utf8()).unwrap_or(1);
            continue;
        }
        return pos;
    }
}

/// Whether evaluating `expr` can have no observable side effect: identifiers, `this`, literals
/// and function expressions. Member accesses are excluded because they may run getters.
pub fn is_side_effect_free(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(_)
        | Expression::ThisExpression(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}

/// Whether the source between `start` and `end` contains a line break.
fn has_line_break(fcx: &FixCtx, start: u32, end: u32) -> bool {
    fcx.source_text()[start as usize..end as usize]
        .chars()
        .any(|ch| matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
}

/// The first significant character at or after `pos`, skipping whitespace and comments.
pub fn next_char(fcx: &FixCtx, pos: u32) -> Option<(u32, char)> {
    let pos = skip_trivia(fcx, pos, false);
    fcx.source_text()[pos as usize..].chars().next().map(|ch| (pos, ch))
}

/// Whether an expression at `node_id` can be replaced by an assignment expression without adding
/// parentheses. Assignments bind more loosely than every operator except `,`, `yield` and arrows,
/// so this is only true where an assignment is allowed as-is.
pub fn assignment_fits_at(fcx: &FixCtx, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;
    let node_span = fcx.nodes().kind(node_id).span();
    match fcx.nodes().parent_kind(node_id) {
        AstKind::ExpressionStatement(_)
        | AstKind::ParenthesizedExpression(_)
        | AstKind::VariableDeclarator(_)
        | AstKind::ReturnStatement(_)
        | AstKind::ThrowStatement(_)
        | AstKind::SequenceExpression(_)
        | AstKind::ArrayExpression(_)
        | AstKind::ObjectProperty(_)
        | AstKind::TemplateLiteral(_)
        | AstKind::SpreadElement(_)
        | AstKind::YieldExpression(_)
        | AstKind::ArrowFunctionExpression(_) => true,
        AstKind::CallExpression(call) => call.callee.span() != node_span,
        AstKind::NewExpression(new) => new.callee.span() != node_span,
        AstKind::AssignmentExpression(assignment) => assignment.right.span() == node_span,
        AstKind::ConditionalExpression(conditional) => conditional.test.span() != node_span,
        _ => false,
    }
}

/// Automatic semicolon insertion hazard check.
///
/// A statement whose source ends at `stmt_end` without a semicolon is about to be followed directly
/// by the code that starts at `rest`. Returns whether that code could join the statement: it starts
/// on the same line, starts with `(`, `[`, `/`, `+`, `-` or a backtick, or the statement ends with
/// `++` or `--`.
pub fn asi_hazard_after(fcx: &FixCtx, stmt_end: u32, rest: u32) -> bool {
    let Some((next, ch)) = next_char(fcx, rest) else { return false };
    let stmt = fcx.source_text()[..stmt_end as usize].trim_end();
    !has_line_break(fcx, rest, next)
        || ASI_HAZARD_START.contains(&ch)
        || stmt.ends_with("++")
        || stmt.ends_with("--")
}

/// How an identifier reference resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// A local binding that may be reassigned (`let`, `var`, parameter, function, class).
    Mutable,
    /// A `const` binding or an import, which throws when assigned.
    Constant,
    /// No local binding: a global, which may have a setter or be read-only.
    Global,
}

pub fn binding_of(fcx: &FixCtx, ident: &oxc_ast::ast::IdentifierReference) -> Binding {
    let Some(symbol_id) = fcx.scoping().get_reference(ident.reference_id()).symbol_id() else {
        return Binding::Global;
    };
    let flags = fcx.scoping().symbol_flags(symbol_id);
    if flags.is_const_variable() || flags.is_import() {
        Binding::Constant
    } else {
        Binding::Mutable
    }
}

/// Whether code inserted at `pos` starts a new statement safely: the previous significant
/// character ends a statement or opens a block, or there is nothing before it.
///
/// A `}` counts only when it closes a block statement or a function or class declaration; after an
/// object literal or a function expression (`x = {}`, `f = function () {}`) the next line can
/// continue the expression. Returns false when a comment precedes `pos`, to stay conservative.
pub fn statement_boundary_before(fcx: &FixCtx, pos: u32) -> bool {
    use oxc_ast::AstKind;
    let before = fcx.source_text()[..pos as usize].trim_end();
    let Some(last) = before.chars().last() else { return true };
    let Ok(last_pos) = u32::try_from(before.len() - last.len_utf8()) else { return false };
    if fcx.ctx.semantic().is_inside_comment(last_pos) {
        return false;
    }
    match last {
        ';' | '{' => true,
        '}' => {
            // The node that owns the `}`: the innermost one ending right after it.
            let owner = fcx
                .innermost(Span::sized(last_pos, 1), |node| node.kind().span().end == last_pos + 1);
            owner.is_some_and(|owner| match owner.kind() {
                AstKind::BlockStatement(_)
                | AstKind::SwitchStatement(_)
                | AstKind::TSModuleBlock(_) => true,
                AstKind::FunctionBody(_) => matches!(
                    fcx.nodes().parent_kind(owner.id()),
                    AstKind::Function(function) if function.is_declaration()
                ),
                AstKind::ClassBody(_) => matches!(
                    fcx.nodes().parent_kind(owner.id()),
                    AstKind::Class(class) if class.is_declaration()
                ),
                _ => false,
            })
        }
        _ => false,
    }
}

/// Characters that continue the previous statement when a line starts with them and that statement
/// has no semicolon.
pub const ASI_HAZARD_START: [char; 6] = ['(', '[', '`', '+', '-', '/'];

/// Whether a statement starting with `text` can be placed at `pos` without joining the previous
/// statement through automatic semicolon insertion.
pub fn safe_statement_start(fcx: &FixCtx, pos: u32, text: &str) -> bool {
    !text.starts_with(ASI_HAZARD_START) || statement_boundary_before(fcx, pos)
}

/// Whether the statement at `span` can be deleted without automatic semicolon insertion joining the
/// code before it with the code after it, as in `foo()\nvar x;\n(bar)()`.
pub fn can_delete_statement(fcx: &FixCtx, span: Span) -> bool {
    statement_boundary_before(fcx, span.start) || !asi_hazard_after(fcx, span.start, span.end)
}

/// Whether the statement at `node_id`, spanning `span`, can be deleted: it sits directly in a
/// statement list, contains no comments, and deleting it creates no ASI hazard.
pub fn can_remove_statement(fcx: &FixCtx, node_id: oxc_semantic::NodeId, span: Span) -> bool {
    is_statement_list(&fcx.nodes().parent_kind(node_id))
        && !fcx.ctx.has_comments_between(span)
        && can_delete_statement(fcx, span)
}

/// Whether the expression at `node_id` starts its enclosing expression statement or arrow body,
/// where a leading `{`, `(`, `function` or `class` changes how the code parses.
pub fn starts_statement(fcx: &FixCtx, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;
    let start = fcx.nodes().kind(node_id).span().start;
    for ancestor in fcx.nodes().ancestor_kinds(node_id) {
        match ancestor {
            AstKind::ExpressionStatement(stmt) => return stmt.span.start == start,
            AstKind::ParenthesizedExpression(_) => return false,
            kind if kind.span().start != start => return false,
            _ => {}
        }
    }
    false
}

/// Whether the expression at `node_id` sits where any expression is allowed (as opposed to an
/// import source, property key, directive or type).
pub fn in_expression_slot(fcx: &FixCtx, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;
    let span = fcx.nodes().kind(node_id).span();
    match fcx.nodes().parent_kind(node_id) {
        AstKind::ExpressionStatement(_)
        | AstKind::VariableDeclarator(_)
        | AstKind::AssignmentExpression(_)
        | AstKind::AssignmentPattern(_)
        | AstKind::CallExpression(_)
        | AstKind::NewExpression(_)
        | AstKind::ArrayExpression(_)
        | AstKind::ReturnStatement(_)
        | AstKind::ThrowStatement(_)
        | AstKind::ParenthesizedExpression(_)
        | AstKind::BinaryExpression(_)
        | AstKind::LogicalExpression(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::TemplateLiteral(_)
        | AstKind::SequenceExpression(_)
        | AstKind::SpreadElement(_)
        | AstKind::AwaitExpression(_)
        | AstKind::YieldExpression(_)
        | AstKind::UnaryExpression(_)
        | AstKind::StaticMemberExpression(_)
        | AstKind::ComputedMemberExpression(_)
        | AstKind::SwitchCase(_)
        | AstKind::ExportDefaultDeclaration(_)
        | AstKind::JSXExpressionContainer(_) => true,
        AstKind::ObjectProperty(prop) => prop.key.span() != span,
        AstKind::PropertyDefinition(prop) => prop.key.span() != span,
        _ => false,
    }
}

/// Whether a `+` concatenation can replace the expression at `node_id` without parentheses.
pub fn concatenation_fits_at(fcx: &FixCtx, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;
    use oxc_syntax::operator::BinaryOperator;
    match fcx.nodes().parent_kind(node_id) {
        AstKind::AssignmentExpression(_)
        | AstKind::AssignmentPattern(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::JSXExpressionContainer(_)
        | AstKind::ExportDefaultDeclaration(_)
        | AstKind::SwitchCase(_) => true,
        AstKind::BinaryExpression(binary) => binary.operator == BinaryOperator::Addition,
        AstKind::ComputedMemberExpression(member) => {
            member.expression.span() == fcx.nodes().kind(node_id).span()
        }
        _ => assignment_fits_at(fcx, node_id),
    }
}

/// Whether the unshadowed global `name` is visible at the scope of `node_id`.
pub fn is_unshadowed_global(fcx: &FixCtx, node_id: oxc_semantic::NodeId, name: &str) -> bool {
    let scope_id = fcx.nodes().get_node(node_id).scope_id();
    fcx.scoping().find_binding(scope_id, name.into()).is_none()
}

/// Whether the expression at `node_id` starts an arrow function's expression body, such as
/// `x` in `() => x.y`. A body starting with `{` would parse as a block.
pub fn starts_arrow_expression_body(fcx: &FixCtx, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;
    let mut child_span = fcx.nodes().kind(node_id).span();
    let start = child_span.start;
    for ancestor in fcx.nodes().ancestor_kinds(node_id) {
        match ancestor {
            AstKind::ArrowFunctionExpression(arrow) => {
                return arrow.get_expression().is_some_and(|body| body.span() == child_span);
            }
            AstKind::ParenthesizedExpression(_) => return false,
            kind if kind.span().start != start => return false,
            kind => child_span = kind.span(),
        }
    }
    false
}

/// Whether `/pattern/flags` is a valid regular expression literal.
pub fn is_valid_regex_literal(pattern: &str, flags: &str) -> bool {
    let allocator = oxc_allocator::Allocator::default();
    let options = oxc_regular_expression::Options { pattern_span_offset: 0, flags_span_offset: 0 };
    oxc_regular_expression::LiteralParser::new(&allocator, pattern, Some(flags), options)
        .parse()
        .is_ok()
}

/// Turns the value of a string passed to `RegExp(..)` into the source of a regex literal with the
/// same meaning: unescaped `/` is escaped, line terminators become escapes, and an empty pattern
/// becomes `(?:)`.
pub fn regex_source_from_string(value: &str) -> String {
    if value.is_empty() {
        return "(?:)".to_string();
    }
    let mut out = String::with_capacity(value.len() + 2);
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                out.push('\\');
                if let Some(next) = chars.next() {
                    push_regex_char(&mut out, next);
                }
            }
            '/' => out.push_str("\\/"),
            _ => push_regex_char(&mut out, ch),
        }
    }
    out
}

fn push_regex_char(out: &mut String, ch: char) {
    match ch {
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\u{2028}' => out.push_str("\\u2028"),
        '\u{2029}' => out.push_str("\\u2029"),
        _ => out.push(ch),
    }
}

/// Whether `expr` can be the operand of a unary operator such as `void` without parentheses.
pub fn binds_as_unary_operand(expr: &Expression) -> bool {
    !matches!(
        expr,
        Expression::BinaryExpression(_)
            | Expression::LogicalExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::YieldExpression(_)
            | Expression::SequenceExpression(_)
    )
}

/// Whether an expression statement starting with `text` would parse as something else: a block,
/// a function or class declaration, or a `let` declaration.
pub fn needs_parens_as_statement(text: &str) -> bool {
    text.starts_with('{')
        || ["function", "class", "let", "async function"]
            .iter()
            .any(|keyword| starts_with_word(text, keyword))
}

/// Whether `ch` continues a word: an alphanumeric character, `_` or `$`.
pub fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

/// Whether `text` starts with the whole word `word`, not just a prefix of a longer identifier.
pub fn starts_with_word(text: &str, word: &str) -> bool {
    text.strip_prefix(word).is_some_and(|rest| !rest.starts_with(is_word_char))
}

/// `text` as the expression of an expression statement, wrapped in parentheses when it would
/// otherwise parse as something else.
pub fn as_statement(text: &str) -> Cow<'_, str> {
    if needs_parens_as_statement(text) {
        Cow::Owned(format!("({text})"))
    } else {
        Cow::Borrowed(text)
    }
}

/// The start of the line containing `pos`, and the spaces and tabs that indent it.
pub fn line_indent(source: &str, pos: u32) -> (u32, &str) {
    let before = &source[..pos as usize];
    let line_start = before.rfind(['\n', '\r']).map_or(0, |i| i + 1);
    let line = &before[line_start..];
    let indent = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
    (u32::try_from(line_start).unwrap_or(0), indent)
}

/// Escapes every unescaped occurrence of `quote` in the raw contents of a string literal.
pub fn escape_unescaped(raw: &str, quote: char) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut escaped = false;
    for ch in raw.chars() {
        if !escaped && ch == quote {
            out.push('\\');
        }
        escaped = !escaped && ch == '\\';
        out.push(ch);
    }
    out
}

/// Whether the raw contents of a string literal contain an escape that is a syntax error in a
/// template literal: a legacy octal escape (`\1`–`\7`, `\0` followed by a digit) or `\8`, `\9`.
pub fn has_legacy_escape(raw: &str) -> bool {
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            continue;
        }
        match chars.next() {
            Some('1'..='9') => return true,
            Some('0') if chars.peek().is_some_and(char::is_ascii_digit) => return true,
            _ => {}
        }
    }
    false
}

/// The value of a string literal or a template literal without substitutions.
pub fn static_string<'b>(argument: &'b Argument) -> Option<&'b str> {
    match argument.as_expression()?.get_inner_expression() {
        Expression::StringLiteral(literal) => literal.value.as_str(),
        Expression::TemplateLiteral(template) => template.single_quasi()?.as_str(),
        _ => None,
    }
}

/// The source text of `arguments`, from the start of the first to the end of the last, or an
/// empty string when there are none.
pub fn arguments_text<'a>(fcx: &FixCtx<'_, 'a>, arguments: &[Argument]) -> &'a str {
    match (arguments.first(), arguments.last()) {
        (Some(first), Some(last)) => {
            fcx.source_range(Span::new(first.span().start, last.span().end))
        }
        _ => "",
    }
}

/// A suggestion that wraps the concise arrow body at `span` in `{}`, so it returns nothing.
pub fn wrap_in_block(fcx: &FixCtx, span: Span) -> crate::fixer::RuleFix {
    fcx.fixer(crate::fixer::FixKind::Suggestion)
        .replace(span, format!("{{{}}}", as_statement(fcx.source_range(span))))
        .with_message("Wrap the expression in `{}`")
}

/// The concise body of the arrow function whose body spans `span`.
pub fn concise_body<'c, 'a>(fcx: &FixCtx<'c, 'a>, span: Span) -> Option<&'c Expression<'a>> {
    use oxc_ast::AstKind;
    let node = fcx.innermost(span, |node| {
        matches!(node.kind(), AstKind::ArrowFunctionExpression(arrow)
            if arrow.get_expression().is_some_and(|body| body.span() == span))
    })?;
    let AstKind::ArrowFunctionExpression(arrow) = node.kind() else { return None };
    arrow.get_expression()
}

/// The descendants of `node_id`: the contiguous run of nodes after it in visit (pre-order) order,
/// ending at the first node whose parent comes before it.
pub fn descendants<'c, 'a>(
    fcx: &FixCtx<'c, 'a>,
    node_id: oxc_semantic::NodeId,
) -> impl Iterator<Item = &'c oxc_semantic::AstNode<'a>> + use<'c, 'a> {
    let nodes = fcx.nodes();
    nodes
        .iter()
        .skip(node_id.index() + 1)
        .take_while(move |node| nodes.parent_id(node.id()) >= node_id)
}

/// Whether `kind` sits directly in a list of statements, where statements can be added or removed.
pub fn is_statement_list(kind: &oxc_ast::AstKind) -> bool {
    use oxc_ast::AstKind;
    matches!(
        kind,
        AstKind::Program(_)
            | AstKind::BlockStatement(_)
            | AstKind::FunctionBody(_)
            | AstKind::StaticBlock(_)
            | AstKind::SwitchCase(_)
    )
}
