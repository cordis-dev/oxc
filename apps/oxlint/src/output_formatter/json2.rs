//! Fork-only (cordis-dev): the `json2` output format.
//!
//! Everything the `github` format reports (end line and column, and `fix`: how the issue can be
//! fixed, see `oxc_linter`'s `fork_fixes/report.rs`), plus `help`, `note`, `url`, `offset` and
//! `length` from the `json` format. Only the first label is reported, as in `github`.
//!
//! It is fast on files with many issues: each file's line starts are indexed once, while the
//! `github` and `json` formats scan the file from the start for every issue. On the React and
//! ESLint repos with every rule on, `github` took 8.3s and `json` 2.8s.
//!
//! Output is one JSON document, with one diagnostic per line, written as each file finishes:
//!
//! ```text
//! {"diagnostics":[
//! {"message":"...","code":"eslint(no-var)","severity":"warning","fix":"safe","file":"a.js","line":1,"column":1,"endLine":1,"endColumn":11,"offset":0,"length":10,"help":"...","url":"..."},
//! ...
//! ],"number_of_files":1,"number_of_rules":10,"threads_count":32,"start_time":0.12}
//! ```
//!
//! Fields that do not apply are left out: `code`, `fix`, `help`, `note` and `url` when there are
//! none, `file` without a source, the positions without a label, and `line` to `endColumn` when the
//! label is outside the source. Lines and columns start at 1 and count bytes, as in `github`.

use std::{borrow::Cow, cell::Cell, rc::Rc};

use rustc_hash::FxHashSet;
use serde::Serialize;

use oxc_diagnostics::{
    Error, LabeledSpan, Severity, SourceCode,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

use super::{InternalFormatter, LintCommandInfo, OutputFormat, json::JsonOutputFormatter};

impl OutputFormat {
    /// Whether the linter should record fix levels for this format.
    pub fn reports_fix_levels(self) -> bool {
        matches!(self, Self::Github | Self::Json2)
    }
}

#[derive(Debug, Default)]
pub struct Json2OutputFormatter {
    /// The reporter wrote `{"diagnostics":[...]`. It does not run when no files are found.
    reported: Rc<Cell<bool>>,
}

impl InternalFormatter for Json2OutputFormatter {
    fn all_rules(&self, enabled_rules: FxHashSet<(&str, &str)>) -> Option<String> {
        JsonOutputFormatter::default().all_rules(enabled_rules)
    }

    /// Closes the document that [`Json2Reporter`] opened.
    fn lint_command_info(&self, info: &LintCommandInfo) -> Option<String> {
        let diagnostics = if self.reported.get() { "" } else { "{\"diagnostics\":[]" };
        let number_of_rules =
            info.number_of_rules.map_or_else(|| "null".to_string(), |n| n.to_string());
        Some(format!(
            "{diagnostics},\"number_of_files\":{},\"number_of_rules\":{number_of_rules},\"threads_count\":{},\"start_time\":{}}}\n",
            info.number_of_files,
            info.threads_count,
            info.start_time.as_secs_f64(),
        ))
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(Json2Reporter { written: false, reported: Rc::clone(&self.reported) })
    }
}

#[derive(Default)]
struct Json2Reporter {
    /// At least one diagnostic, and so `{"diagnostics":[`, was written.
    written: bool,
    reported: Rc<Cell<bool>>,
}

impl Json2Reporter {
    fn write(&mut self, out: &mut Vec<u8>, error: &Error, lines: Option<&LineIndex<'_>>) {
        out.extend_from_slice(if self.written { b",\n" } else { b"{\"diagnostics\":[\n" });
        self.written = true;
        serde_json::to_writer(out, &Diagnostic::new(error, lines))
            .expect("serializing to a Vec cannot fail");
    }
}

impl DiagnosticReporter for Json2Reporter {
    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        self.reported.set(true);
        Some(if self.written { "\n]" } else { "{\"diagnostics\":[]" }.to_string())
    }

    fn supports_minified_file_fallback(&self) -> bool {
        false
    }

    /// The linter sends one file's diagnostics at a time, so each source is indexed once. Only
    /// diagnostics with a label need the index.
    fn render_errors(&mut self, errors: Vec<Error>, emit: &mut dyn FnMut(&str)) {
        let mut out = Vec::new();
        let mut lines: Option<LineIndex<'_>> = None;
        for error in &errors {
            let source = error.source_code().map(SourceCode::data);
            if let Some(data) = source
                && !error.labels().is_empty()
                && !lines.as_ref().is_some_and(|lines| lines.is_for(data))
            {
                lines = Some(LineIndex::new(data));
            }
            self.write(&mut out, error, source.and(lines.as_ref()));
        }
        emit(std::str::from_utf8(&out).expect("serde_json writes UTF-8"));
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let mut rendered = String::new();
        self.render_errors(vec![error], &mut |output| rendered.push_str(output));
        Some(rendered)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic<'a> {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<Cow<'a, str>>,
    severity: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<Cow<'a, str>>,
}

impl<'a> Diagnostic<'a> {
    /// `lines` indexes `error`'s source, if it has one.
    fn new(error: &'a Error, lines: Option<&LineIndex<'_>>) -> Self {
        let severity = match error.severity() {
            Some(Severity::Error) | None => "error",
            Some(Severity::Warning) => "warning",
            Some(Severity::Advice) => "advice",
        };
        let label = error.labels().first();
        let (offset, length) = (label.map(LabeledSpan::offset), label.map(LabeledSpan::len));
        let (start, end) = match (lines, label) {
            (Some(lines), Some(label)) => lines.span(label.offset(), label.len()).unzip(),
            _ => (None, None),
        };
        Self {
            message: error.to_string(),
            code: error.code(),
            severity,
            fix: error.fix_level(),
            file: error.source_code().and_then(|source| source.name()),
            line: start.map(|p| p.0),
            column: start.map(|p| p.1),
            end_line: end.map(|p| p.0),
            end_column: end.map(|p| p.1),
            offset,
            length,
            help: error.help(),
            note: error.note(),
            url: error.url(),
        }
    }
}

/// 1-based `(line, column)`.
type Position = (usize, usize);

/// Where each line of a source starts. Line breaks are `\n`, `\r\n` and a lone `\r`, as in the
/// `github` format.
struct LineIndex<'a> {
    data: &'a [u8],
    starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    fn new(data: &'a [u8]) -> Self {
        let mut starts = vec![0];
        for (index, &byte) in data.iter().enumerate() {
            match byte {
                b'\n' => starts.push(index + 1),
                b'\r' if data.get(index + 1) != Some(&b'\n') => starts.push(index + 1),
                _ => {}
            }
        }
        Self { data, starts }
    }

    fn is_for(&self, data: &[u8]) -> bool {
        std::ptr::eq(self.data, data)
    }

    /// Positions of the span's start and end, or none if the span is outside the source.
    fn span(&self, offset: u32, length: u32) -> Option<(Position, Position)> {
        let (start, end) = (offset as usize, offset as usize + length as usize);
        (end <= self.data.len()).then(|| (self.position(start), self.position(end)))
    }

    /// Requires `offset <= data.len()`. An offset between `\r` and `\n` counts as the `\r`.
    fn position(&self, mut offset: usize) -> Position {
        if offset > 0 && self.data[offset - 1] == b'\r' && self.data.get(offset) == Some(&b'\n') {
            offset -= 1;
        }
        let line = self.starts.partition_point(|&start| start <= offset);
        (line, offset - self.starts[line - 1] + 1)
    }
}

#[cfg(test)]
pub(super) mod fixable {
    use std::{borrow::Cow, fmt};

    use oxc_diagnostics::{Diagnostic, Error, LabeledSpan, Severity, SourceCode};

    /// A diagnostic with a fix level, as the linter reports for `json2` and `github`.
    #[derive(Debug)]
    pub struct Fixable(pub Error, pub &'static str);

    impl fmt::Display for Fixable {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    impl std::error::Error for Fixable {}

    impl Diagnostic for Fixable {
        fn code(&self) -> Option<Cow<'_, str>> {
            self.0.code()
        }
        fn severity(&self) -> Option<Severity> {
            self.0.severity()
        }
        fn help(&self) -> Option<Cow<'_, str>> {
            self.0.help()
        }
        fn fix_level(&self) -> Option<&'static str> {
            Some(self.1)
        }
        fn source_code(&self) -> Option<&dyn SourceCode> {
            self.0.source_code()
        }
        fn labels(&self) -> &[LabeledSpan] {
            self.0.labels()
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use oxc_diagnostics::{
        DiagnosticService, NamedSource, OxcDiagnostic, reporter::DiagnosticReporter,
    };
    use oxc_span::Span;

    use super::{Json2OutputFormatter, Json2Reporter, LineIndex, fixable::Fixable};
    use crate::{
        output_formatter::{InternalFormatter, LintCommandInfo, OxlintSuppressionFileAction},
        tester::Tester,
    };

    /// What the reporter writes for `errors`.
    fn render(errors: Vec<oxc_diagnostics::Error>) -> String {
        run(&Json2OutputFormatter::default(), errors)
    }

    /// The whole document: what the reporter writes, then the closing summary.
    fn render_document(errors: Vec<oxc_diagnostics::Error>) -> String {
        let formatter = Json2OutputFormatter::default();
        let output = run(&formatter, errors);
        let info = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 2,
            number_of_rules: None,
            threads_count: 4,
            start_time: Duration::from_millis(1500),
            oxlint_suppression_file_action: OxlintSuppressionFileAction::None,
            rule_timings: None,
        });
        output + &info.unwrap()
    }

    fn run(formatter: &Json2OutputFormatter, errors: Vec<oxc_diagnostics::Error>) -> String {
        let (mut service, sender) = DiagnosticService::new(formatter.get_diagnostic_reporter());
        sender.send(errors).unwrap();
        drop(sender);
        let mut output = Vec::new();
        service.run(&mut output);
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn reports_github_and_json_fields() {
        let source = NamedSource::new("dir/a.js", "let a;\r\nvar b = 1;\n");
        let fixable = OxcDiagnostic::warn("Unexpected var")
            .with_error_code("eslint", "no-var")
            .with_label(Span::new(8, 18))
            .with_source_code(source.clone());
        let plain =
            OxcDiagnostic::error("Bad \"a\"").with_label(Span::new(4, 5)).with_source_code(source);

        let output = render_document(vec![Box::new(Fixable(fixable, "safe")), plain]);

        assert_eq!(
            output,
            r#"{"diagnostics":[
{"message":"Unexpected var","code":"eslint(no-var)","severity":"warning","fix":"safe","file":"dir/a.js","line":2,"column":1,"endLine":2,"endColumn":11,"offset":8,"length":10},
{"message":"Bad \"a\"","severity":"error","file":"dir/a.js","line":1,"column":5,"endLine":1,"endColumn":6,"offset":4,"length":1}
],"number_of_files":2,"number_of_rules":null,"threads_count":4,"start_time":1.5}
"#
        );
        serde_json::from_str::<serde_json::Value>(&output).unwrap();
    }

    #[test]
    fn help_note_and_url_are_kept() {
        let error = OxcDiagnostic::warn("m")
            .with_help("h")
            .with_note("n")
            .with_url("https://example.com")
            .with_label(Span::new(0, 1))
            .with_source_code(NamedSource::new("a.js", "a"));

        let output = Json2Reporter::default().render_error(error).unwrap();

        assert_eq!(
            output,
            r#"{"diagnostics":[
{"message":"m","severity":"warning","file":"a.js","line":1,"column":1,"endLine":1,"endColumn":2,"offset":0,"length":1,"help":"h","note":"n","url":"https://example.com"}"#
        );
    }

    #[test]
    fn no_diagnostics_is_valid_json() {
        let output = render_document(vec![]);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&output).unwrap()["diagnostics"],
            serde_json::json!([])
        );
    }

    #[test]
    fn without_source_or_label_positions_are_left_out() {
        let fileless = OxcDiagnostic::warn("no file").with_label(Span::new(0, 1));
        let unlabeled =
            OxcDiagnostic::warn("no label").with_source_code(NamedSource::new("a.js", "a"));
        let outside = OxcDiagnostic::warn("outside")
            .with_label(Span::new(5, 6))
            .with_source_code(NamedSource::new("a.js", "a"));

        let output = render(vec![fileless.into(), unlabeled, outside]);

        assert_eq!(
            output,
            r#"{"diagnostics":[
{"message":"no file","severity":"warning","offset":0,"length":1},
{"message":"no label","severity":"warning","file":"a.js"},
{"message":"outside","severity":"warning","file":"a.js","offset":5,"length":1}
]"#
        );
    }

    #[test]
    fn many_files_are_indexed_separately() {
        let a = OxcDiagnostic::warn("a")
            .with_label(Span::new(2, 3))
            .with_source_code(NamedSource::new("a.js", "\n\na"));
        let b = OxcDiagnostic::warn("b")
            .with_label(Span::new(0, 1))
            .with_source_code(NamedSource::new("b.js", "b"));

        let output = render(vec![a, b]);

        assert!(output.contains(r#""file":"a.js","line":3,"column":1"#), "{output}");
        assert!(output.contains(r#""file":"b.js","line":1,"column":1"#), "{output}");
    }

    #[test]
    fn positions_match_the_github_format() {
        let lines = LineIndex::new(b"a\r\nb\rc\nd");
        // start of each line
        assert_eq!(lines.position(0), (1, 1));
        assert_eq!(lines.position(3), (2, 1));
        assert_eq!(lines.position(5), (3, 1));
        assert_eq!(lines.position(7), (4, 1));
        // between `\r` and `\n` counts as the `\r`
        assert_eq!(lines.position(2), (1, 2));
        assert_eq!(lines.position(1), (1, 2));
        // end of input
        assert_eq!(lines.position(8), (4, 2));
    }

    /// The whole CLI output is one JSON document, also when no files are found.
    #[test]
    fn cli_output_is_valid_json() {
        let tester = Tester::new().with_cwd("fixtures/cli/output_formatter_diagnostic".into());
        for args in [
            &["--format=json2", "test.js"][..],
            &["--format=json2", "ok.js"],
            &["--format=json2", "--no-error-on-unmatched-pattern", "missing.js"],
        ] {
            let output = tester.test_output_verbose(args);
            let json: serde_json::Value = serde_json::from_str(&output)
                .unwrap_or_else(|error| panic!("{args:?}: {error}\n{output}"));
            assert!(json["diagnostics"].is_array(), "{args:?}: {output}");
            assert!(json["number_of_files"].is_number(), "{args:?}: {output}");
        }
    }

    /// Same start and end as `Info`, which the `github` format uses, at every offset.
    #[test]
    fn positions_match_info() {
        let source = "a\r\nb\rc\n\nd\r\r\n\u{e9}x\r";
        let lines = LineIndex::new(source.as_bytes());
        let len = u32::try_from(source.len()).unwrap();
        for start in 0..=len {
            for end in start..=len {
                let error = OxcDiagnostic::warn("m")
                    .with_label(Span::new(start, end))
                    .with_source_code(NamedSource::new("a.js", source));
                let info = oxc_diagnostics::reporter::Info::new(&error);
                let (json2_start, json2_end) = lines.span(start, end - start).unwrap();
                assert_eq!(
                    json2_start,
                    (info.start.line, info.start.column),
                    "start {start}..{end}"
                );
                assert_eq!(json2_end, (info.end.line, info.end.column), "end {start}..{end}");
            }
        }
    }
}
