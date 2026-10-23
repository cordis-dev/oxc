use oxc_semantic::{AstNode, AstNodes, Scoping};
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{FixKind, Message, RuleFixer},
};

/// What a fork fixer sees: the lint context and the diagnostic it is fixing.
///
/// The diagnostic is read-only. Fixers must not try to change its message or help text.
pub struct FixCtx<'c, 'a> {
    pub ctx: &'c LintContext<'a>,
    pub message: &'c Message,
}

impl<'c, 'a> FixCtx<'c, 'a> {
    pub(super) fn new(ctx: &'c LintContext<'a>, message: &'c Message) -> Self {
        Self { ctx, message }
    }

    /// The span of the diagnostic's primary label.
    pub fn span(&self) -> Span {
        self.message.span
    }

    /// Spans of every label on the diagnostic, in order.
    pub fn label_spans(&self) -> Vec<Span> {
        self.message.error.labels.iter().map(label_to_span).collect()
    }

    /// The span of the diagnostic's label at `index`.
    pub fn label_span(&self, index: usize) -> Option<Span> {
        self.message.error.labels.get(index).map(label_to_span)
    }

    /// The diagnostic's message text.
    pub fn text(&self) -> &str {
        &self.message.error.message
    }

    /// The diagnostic's help text, if any.
    pub fn help(&self) -> Option<&str> {
        self.message.error.help.as_deref()
    }

    /// A fixer whose factory methods produce fixes of `kind`.
    pub fn fixer(&self, kind: FixKind) -> RuleFixer<'c, 'a> {
        RuleFixer::new(kind, self.ctx)
    }

    pub fn nodes(&self) -> &'c AstNodes<'a> {
        self.ctx.semantic().nodes()
    }

    pub fn scoping(&self) -> &'c Scoping {
        self.ctx.semantic().scoping()
    }

    pub fn source_text(&self) -> &'a str {
        self.ctx.source_text()
    }

    pub fn source_range(&self, span: Span) -> &'a str {
        self.ctx.source_range(span)
    }

    /// The innermost node whose span contains `span` and that matches `pred`.
    pub fn innermost(
        &self,
        span: Span,
        pred: impl Fn(&AstNode<'a>) -> bool,
    ) -> Option<&'c AstNode<'a>> {
        super::nodes::innermost(self.nodes(), span, pred)
    }

    /// The innermost node whose span equals `span` and that matches `pred`.
    pub fn exact(
        &self,
        span: Span,
        pred: impl Fn(&AstNode<'a>) -> bool,
    ) -> Option<&'c AstNode<'a>> {
        self.innermost(span, |node| oxc_span::GetSpan::span(&node.kind()) == span && pred(node))
    }
}

fn label_to_span(label: &oxc_diagnostics::LabeledSpan) -> Span {
    Span::new(label.offset(), label.offset() + label.len())
}
