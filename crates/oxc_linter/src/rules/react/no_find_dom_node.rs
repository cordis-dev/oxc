use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_find_dom_node_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected call to `findDOMNode`.")
        .with_help("Replace `findDOMNode` with one of the alternatives documented at https://react.dev/reference/react-dom/findDOMNode#alternatives.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoFindDomNode;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows the use of `findDOMNode`.
    ///
    /// ### Why is this bad?
    ///
    /// `findDOMNode` is an escape hatch used to access the underlying DOM node.
    /// In most cases, use of this escape hatch is discouraged because it pierces the component abstraction.
    /// [It has been deprecated in `StrictMode`.](https://legacy.reactjs.org/docs/strict-mode.html#warning-about-deprecated-finddomnode-usage)
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// class MyComponent extends Component {
    ///   componentDidMount() {
    ///     findDOMNode(this).scrollIntoView();
    ///   }
    ///   render() {
    ///     return <div />;
    ///   }
    /// }
    /// ```
    NoFindDomNode,
    react,
    correctness
);

impl Rule for NoFindDomNode {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if let Some(ident) = call_expr.callee.get_identifier_reference() {
            if ident.name == "findDOMNode" {
                ctx.diagnostic(no_find_dom_node_diagnostic(ident.span));
            }
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        let member = member_expr.object();
        if !member.is_specific_id("React")
            && !member.is_specific_id("ReactDOM")
            && !member.is_specific_id("ReactDom")
        {
            return;
        }
        let Some((span, "findDOMNode")) = member_expr.static_property_info() else {
            return;
        };
        ctx.diagnostic(no_find_dom_node_diagnostic(span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var Hello = function() {};", None),
        (
            r"
            var Hello = createReactClass({
              render: function() {
                return <div>Hello</div>;
              }
            });
            ",
            None,
        ),
        (
            r"
            var Hello = createReactClass({
              componentDidMount: function() {
                someNonMemberFunction(arg);
                this.someFunc = React.findDOMNode;
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            ",
            None,
        ),
        (
            r"
            var Hello = createReactClass({
              componentDidMount: function() {
                React.someFunc(this);
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            ",
            None,
        ),
        (
            r"
            var Hello = createReactClass({
              componentDidMount: function() {
                SomeModule.findDOMNode(this).scrollIntoView();
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            r"
            var Hello = createReactClass({
              componentDidMount: function() {
                React.findDOMNode(this).scrollIntoView();
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            ",
            None,
        ),
        (
            r"
            var Hello = createReactClass({
              componentDidMount: function() {
                ReactDOM.findDOMNode(this).scrollIntoView();
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            ",
            None,
        ),
        (
            r"
            var Hello = createReactClass({
              componentDidMount: function() {
                ReactDom.findDOMNode(this).scrollIntoView();
              },
              render: function() {
                return <div>Hello</div>;
              }
            });
            ",
            None,
        ),
        (
            r"
            class Hello extends Component {
              componentDidMount() {
                findDOMNode(this).scrollIntoView();
              }
              render() {
                return <div>Hello</div>;
              }
            }
            ",
            None,
        ),
        (
            r"
            class Hello extends Component {
              componentDidMount() {
                this.node = findDOMNode(this);
              }
              render() {
                return <div>Hello</div>;
              }
            }
            ",
            None,
        ),
    ];

    Tester::new(NoFindDomNode::NAME, NoFindDomNode::PLUGIN, pass, fail).test_and_snapshot();
}
