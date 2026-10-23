//! Span-to-node lookup.
//!
//! Diagnostics carry spans, not nodes. Node spans nest: two nodes either contain one another or do
//! not overlap. So all nodes containing a span lie on one ancestor chain, and the deepest of them is
//! an ancestor of (or equal to) the node that starts last at or before the span. A per-thread index
//! of node spans sorted by start finds that node by binary search; walking up the real parent links
//! from there finds the innermost node that contains the span and matches the predicate.
//!
//! The index is cached per thread for the file being linted (a file is linted on one thread). Every
//! lookup checks the indexed node against the live AST and rebuilds the index on a mismatch, so a
//! stale cache can never return a node from another file.

use std::cell::RefCell;

use oxc_semantic::{AstNode, AstNodes, NodeId};
use oxc_span::{GetSpan, Span};

/// Identifies the `AstNodes` of one file, for caches that live across diagnostics.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FileKey {
    nodes_addr: usize,
    len: usize,
    first: Span,
    last: Span,
}

struct Index {
    key: FileKey,
    /// `(start, end, id)` sorted by start ascending, then end descending, so that among nodes with
    /// the same start the innermost comes last.
    entries: Vec<(u32, u32, NodeId)>,
}

thread_local! {
    static INDEX: RefCell<Option<Index>> = const { RefCell::new(None) };
}

pub fn file_key(nodes: &AstNodes) -> FileKey {
    let span_at =
        |index: usize| nodes.iter().nth(index).map(|node| node.kind().span()).unwrap_or_default();
    FileKey {
        nodes_addr: std::ptr::from_ref(nodes) as usize,
        len: nodes.len(),
        first: span_at(0),
        last: if nodes.is_empty() { Span::default() } else { span_at(nodes.len() - 1) },
    }
}

fn build(nodes: &AstNodes, key: FileKey) -> Index {
    let mut entries: Vec<(u32, u32, NodeId)> = nodes
        .iter()
        .map(|node| {
            let span = node.kind().span();
            (span.start, span.end, node.id())
        })
        .collect();
    entries.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
    Index { key, entries }
}

/// The node that starts last at or before `span.start`, taken from the index.
fn latest_start(nodes: &AstNodes, span: Span) -> Option<NodeId> {
    let key = file_key(nodes);
    INDEX.with_borrow_mut(|index| {
        for attempt in 0..2 {
            if attempt == 1 || index.as_ref().is_none_or(|index| index.key != key) {
                *index = Some(build(nodes, key));
            }
            let entries = &index.as_ref().unwrap().entries;
            let position = entries.partition_point(|entry| entry.0 <= span.start);
            let &(start, end, id) = entries.get(position.checked_sub(1)?)?;
            if nodes.kind(id).span() == Span::new(start, end) {
                return Some(id);
            }
        }
        None
    })
}

pub(super) fn innermost<'n, 'a>(
    nodes: &'n AstNodes<'a>,
    span: Span,
    pred: impl Fn(&AstNode<'a>) -> bool,
) -> Option<&'n AstNode<'a>> {
    let contains = |node: &AstNode| {
        let node_span = node.kind().span();
        node_span.start <= span.start && span.end <= node_span.end
    };
    let mut id = latest_start(nodes, span)?;
    loop {
        let node = nodes.get_node(id);
        if contains(node) && pred(node) {
            return Some(node);
        }
        let parent = nodes.parent_id(id);
        if parent == id {
            return None;
        }
        id = parent;
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::{AstNode, SemanticBuilder};
    use oxc_span::{GetSpan, SourceType, Span};

    /// The index-based lookup must agree with a plain scan for every node's span.
    #[test]
    fn index_matches_linear_scan() {
        let source = "function f(a = 1) { if (a) { return [a, { b: `x${a}y` }]; } }\nclass C { #p = 1; m() { return this.#p; } }\nlabel: for (const x of y) { x?.z?.(1); }";
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();
        let semantic = SemanticBuilder::new().build(&ret.program).semantic;
        let nodes = semantic.nodes();
        let linear = |span: Span| {
            nodes.iter().fold(None::<&AstNode>, |found, node| {
                let s = node.kind().span();
                if s.start <= span.start && span.end <= s.end { Some(node) } else { found }
            })
        };
        for node in nodes.iter() {
            let span = node.kind().span();
            let expected = linear(span).map(AstNode::id);
            let actual = super::innermost(nodes, span, |_| true).map(AstNode::id);
            assert_eq!(actual, expected, "span {span:?}");
        }
        for offset in 0..u32::try_from(source.len()).unwrap() {
            let span = Span::new(offset, offset);
            assert_eq!(
                super::innermost(nodes, span, |_| true).map(AstNode::id),
                linear(span).map(AstNode::id),
                "offset {offset}"
            );
        }
    }
}
