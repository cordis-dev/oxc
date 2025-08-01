//! ECMAScript Token Kinds

use std::fmt::{self, Display};

/// Lexer token kind
///
/// Exported for other oxc crates to use. You generally don't need to use this directly.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Kind {
    #[default]
    Eof = 0,
    Undetermined,
    Skip, // Whitespace, line breaks, comments
    // 12.5 Hashbang Comments
    HashbangComment,
    // 12.7.1 identifier
    Ident,
    // 12.7.2 keyword
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    New,
    Return,
    Super,
    Switch,
    This,
    Throw,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    // Contextual Keywords
    Async,
    From,
    Get,
    Meta, // import.meta
    Of,
    Set,
    Target,   // new.target
    Accessor, // keyword from https://github.com/tc39/proposal-decorators
    Source,   // import.source https://github.com/tc39/proposal-source-phase-imports
    Defer,    // import.defer https://github.com/tc39/proposal-defer-import-eval
    // TypeScript Contextual Keywords
    Abstract,
    As,
    Asserts,
    Assert,
    Any,
    Boolean,
    Constructor,
    Declare,
    Infer,
    Intrinsic,
    Is,
    KeyOf,
    Module,
    Namespace,
    Never,
    Out,
    Readonly,
    Require,
    Number, // the "number" keyword for TypeScript
    Object,
    Satisfies,
    String, // the "string" keyword for TypeScript
    Symbol,
    Type,
    Undefined,
    Unique,
    Using,
    Unknown,
    Global,
    BigInt, // the "bigint" keyword for TypeScript
    Override,
    // Future keywords (strict mode reserved words)
    Implements,
    Interface,
    Let,
    Package,
    Private,
    Protected,
    Public,
    Static,
    Yield,
    // 12.8 punctuators
    Amp, // &
    Amp2,
    Amp2Eq,
    AmpEq,
    Bang, // !
    Caret,
    CaretEq,
    Colon,
    Comma,
    Dot,
    Dot3, // ...
    Eq,
    Eq2,
    Eq3,
    GtEq, // >=
    LAngle,
    LBrack,
    LCurly,
    LParen,
    LtEq, // <=
    Minus,
    Minus2,
    MinusEq,
    Neq,
    Neq2,
    Percent,
    PercentEq,
    Pipe,
    Pipe2,
    Pipe2Eq,
    PipeEq,
    Plus,
    Plus2,
    PlusEq,
    Question,
    Question2,
    Question2Eq,
    QuestionDot,
    RAngle,
    RBrack,
    RCurly,
    RParen,
    Semicolon,
    ShiftLeft,     // <<
    ShiftLeftEq,   // <<=
    ShiftRight,    // >>
    ShiftRight3,   // >>>
    ShiftRight3Eq, // >>>=
    ShiftRightEq,  // >>=
    Slash,
    SlashEq,
    Star,
    Star2,
    Star2Eq,
    StarEq,
    Tilde,
    // arrow function
    Arrow,
    // 12.9.1 Null Literals
    Null,
    // 12.9.2 Boolean Literals
    True,
    False,
    // 12.9.3 Numeric Literals
    Decimal,
    Float,
    Binary,
    Octal,
    Hex,
    // for `1e10`, `1e+10`
    PositiveExponential,
    // for `1e-10`
    NegativeExponential,
    // 12.9.4 String Literals
    /// String Type
    Str,
    // 12.9.5 Regular Expression Literals
    RegExp,
    // 12.9.6 Template Literal
    NoSubstitutionTemplate,
    TemplateHead,
    TemplateMiddle,
    TemplateTail,
    // es2022 Private Identifier
    PrivateIdentifier,
    // JSX
    JSXText,
    // Decorator
    At,
}

#[allow(clippy::enum_glob_use, clippy::allow_attributes)]
use Kind::*;

impl Kind {
    #[inline]
    pub fn is_eof(self) -> bool {
        self == Eof
    }

    #[inline]
    pub fn is_number(self) -> bool {
        matches!(
            self,
            Float | Decimal | Binary | Octal | Hex | PositiveExponential | NegativeExponential
        )
    }

    #[inline] // Inline into `read_non_decimal` - see comment there as to why
    pub fn matches_number_byte(self, b: u8) -> bool {
        match self {
            Decimal => b.is_ascii_digit(),
            Binary => matches!(b, b'0'..=b'1'),
            Octal => matches!(b, b'0'..=b'7'),
            Hex => b.is_ascii_hexdigit(),
            _ => unreachable!(),
        }
    }

    /// [Identifiers](https://tc39.es/ecma262/#sec-identifiers)
    /// `IdentifierReference`
    #[inline]
    pub fn is_identifier_reference(self, is_yield_context: bool, is_await_context: bool) -> bool {
        self.is_identifier()
            || (!is_yield_context && self == Yield)
            || (!is_await_context && self == Await)
    }

    /// `BindingIdentifier`
    #[inline]
    pub fn is_binding_identifier(self) -> bool {
        self.is_identifier() || matches!(self, Yield | Await)
    }

    /// `LabelIdentifier`
    #[inline]
    pub fn is_label_identifier(self, is_yield_context: bool, is_await_context: bool) -> bool {
        self.is_identifier()
            || (!is_yield_context && self == Yield)
            || (!is_await_context && self == Await)
    }

    /// Identifier
    /// `IdentifierName` but not `ReservedWord`
    #[inline]
    pub fn is_identifier(self) -> bool {
        self.is_identifier_name() && !self.is_reserved_keyword()
    }

    /// TypeScript Identifier
    ///
    /// <https://github.com/microsoft/TypeScript/blob/15392346d05045742e653eab5c87538ff2a3c863/src/compiler/parser.ts#L2316-L2335>
    #[inline]
    pub fn is_ts_identifier(self, is_yield_context: bool, is_await_context: bool) -> bool {
        self.is_identifier_reference(is_yield_context, is_await_context)
            && !self.is_strict_mode_contextual_keyword()
            && !self.is_contextual_keyword()
    }

    /// `IdentifierName`
    #[inline]
    pub fn is_identifier_name(self) -> bool {
        self == Ident || self.is_any_keyword()
    }

    /// Check the succeeding token of a `let` keyword.
    ///
    /// ```javascript
    /// let { a, b } = c, let [a, b] = c, let ident
    /// ```
    #[inline]
    pub fn is_after_let(self) -> bool {
        !matches!(self, In | Instanceof)
            && (matches!(self, LCurly | LBrack | Ident) || self.is_any_keyword())
    }

    /// Section 13.2.4 Literals
    /// Literal :
    ///     `NullLiteral`
    ///     `BooleanLiteral`
    ///     `NumericLiteral`
    ///     `StringLiteral`
    #[inline]
    pub fn is_literal(self) -> bool {
        matches!(self, Null | True | False | Str | RegExp) || self.is_number()
    }

    #[inline]
    pub fn is_after_await_or_yield(self) -> bool {
        !self.is_binary_operator() && (self.is_literal() || self.is_identifier_name())
    }

    /// Section 13.2.6 Object Initializer
    /// `LiteralPropertyName` :
    ///     `IdentifierName`
    ///     `StringLiteral`
    ///     `NumericLiteral`
    #[inline]
    pub fn is_literal_property_name(self) -> bool {
        self.is_identifier_name() || self == Str || self.is_number()
    }

    #[inline]
    pub fn is_identifier_or_keyword(self) -> bool {
        self.is_literal_property_name() || self == Self::PrivateIdentifier
    }

    #[inline]
    pub fn is_variable_declaration(self) -> bool {
        matches!(self, Var | Let | Const)
    }

    #[rustfmt::skip]
    #[inline]
    pub fn is_assignment_operator(self) -> bool {
        matches!(
            self,
            Eq | PlusEq | MinusEq | StarEq | SlashEq | PercentEq | ShiftLeftEq | ShiftRightEq
            | ShiftRight3Eq | Pipe2Eq | Amp2Eq | PipeEq | CaretEq | AmpEq | Question2Eq | Star2Eq
        )
    }

    #[rustfmt::skip]
    #[inline]
    pub fn is_binary_operator(self) -> bool {
        matches!(
            self,
            Eq2 | Neq | Eq3 | Neq2 | LAngle | LtEq | RAngle | GtEq | ShiftLeft | ShiftRight | ShiftRight3
            | Plus | Minus | Star | Slash | Percent | Pipe | Caret | Amp | In | Instanceof | Star2
        )
    }

    #[inline]
    pub fn is_logical_operator(self) -> bool {
        matches!(self, Pipe2 | Amp2 | Question2)
    }

    #[inline]
    pub fn is_unary_operator(self) -> bool {
        matches!(self, Minus | Plus | Bang | Tilde | Typeof | Void | Delete)
    }

    #[inline]
    pub fn is_update_operator(self) -> bool {
        matches!(self, Plus2 | Minus2)
    }

    /// [Keywords and Reserved Words](https://tc39.es/ecma262/#sec-keywords-and-reserved-words)
    #[inline]
    pub fn is_any_keyword(self) -> bool {
        self.is_reserved_keyword()
            || self.is_contextual_keyword()
            || self.is_strict_mode_contextual_keyword()
            || self.is_future_reserved_keyword()
    }

    #[rustfmt::skip]
    #[inline]
    pub fn is_reserved_keyword(self) -> bool {
        matches!(
            self,
            Await | Break | Case | Catch | Class | Const | Continue | Debugger | Default
            | Delete | Do | Else | Enum | Export | Extends | False | Finally | For | Function | If
            | Import | In | Instanceof | New | Null | Return | Super | Switch | This | Throw
            | True | Try | Typeof | Var | Void | While | With | Yield
        )
    }

    #[rustfmt::skip]
    #[inline]
    pub fn is_strict_mode_contextual_keyword(self) -> bool {
        matches!(self, Let | Static | Implements | Interface | Package | Private | Protected | Public)
    }

    #[rustfmt::skip]
    #[inline]
    pub fn is_contextual_keyword(self) -> bool {
        matches!(
            self,
            Async | From | Get | Meta | Of | Set | Target | Accessor | Abstract | As | Asserts | Assert
            | Any | Boolean | Constructor | Declare | Infer | Intrinsic | Is | KeyOf | Module | Namespace
            | Never | Out | Readonly | Require | Number | Object | Satisfies | String | Symbol | Type
            | Undefined | Unique | Unknown | Using | Global | BigInt | Override | Source | Defer
        )
    }

    #[rustfmt::skip]
    #[inline]
    pub fn is_future_reserved_keyword(self) -> bool {
        matches!(self, Implements | Interface | Package | Private | Protected | Public | Static)
    }

    #[inline]
    pub fn is_template_start_of_tagged_template(self) -> bool {
        matches!(self, NoSubstitutionTemplate | TemplateHead)
    }

    #[rustfmt::skip]
    #[inline]
    pub fn is_modifier_kind(self) -> bool {
        matches!(
            self,
            Abstract | Accessor | Async | Const | Declare
            | In | Out | Public | Private | Protected | Readonly | Static | Override
        )
    }

    #[inline]
    pub fn is_binding_identifier_or_private_identifier_or_pattern(self) -> bool {
        matches!(self, LCurly | LBrack | PrivateIdentifier) || self.is_binding_identifier()
    }

    pub fn match_keyword(s: &str) -> Self {
        let len = s.len();
        if len <= 1 || len >= 12 || !s.as_bytes()[0].is_ascii_lowercase() {
            return Ident;
        }
        Self::match_keyword_impl(s)
    }

    fn match_keyword_impl(s: &str) -> Self {
        match s {
            "as" => As,
            "do" => Do,
            "if" => If,
            "in" => In,
            "is" => Is,
            "of" => Of,

            "any" => Any,
            "for" => For,
            "get" => Get,
            "let" => Let,
            "new" => New,
            "out" => Out,
            "set" => Set,
            "try" => Try,
            "var" => Var,

            "case" => Case,
            "else" => Else,
            "enum" => Enum,
            "from" => From,
            "meta" => Meta,
            "null" => Null,
            "this" => This,
            "true" => True,
            "type" => Type,
            "void" => Void,
            "with" => With,

            "async" => Async,
            "await" => Await,
            "break" => Break,
            "catch" => Catch,
            "class" => Class,
            "const" => Const,
            "false" => False,
            "infer" => Infer,
            "keyof" => KeyOf,
            "never" => Never,
            "super" => Super,
            "throw" => Throw,
            "using" => Using,
            "while" => While,
            "yield" => Yield,
            "defer" => Defer,

            "assert" => Assert,
            "bigint" => BigInt,
            "delete" => Delete,
            "export" => Export,
            "global" => Global,
            "import" => Import,
            "module" => Module,
            "number" => Number,
            "object" => Object,
            "public" => Public,
            "return" => Return,
            "static" => Static,
            "string" => String,
            "switch" => Switch,
            "symbol" => Symbol,
            "target" => Target,
            "typeof" => Typeof,
            "unique" => Unique,
            "source" => Source,

            "asserts" => Asserts,
            "boolean" => Boolean,
            "declare" => Declare,
            "default" => Default,
            "extends" => Extends,
            "finally" => Finally,
            "package" => Package,
            "private" => Private,
            "require" => Require,
            "unknown" => Unknown,

            "abstract" => Abstract,
            "accessor" => Accessor,
            "continue" => Continue,
            "debugger" => Debugger,
            "function" => Function,
            "override" => Override,
            "readonly" => Readonly,

            "interface" => Interface,
            "intrinsic" => Intrinsic,
            "namespace" => Namespace,
            "protected" => Protected,
            "satisfies" => Satisfies,
            "undefined" => Undefined,

            "implements" => Implements,
            "instanceof" => Instanceof,

            "constructor" => Constructor,
            _ => Ident,
        }
    }

    pub fn to_str(self) -> &'static str {
        #[expect(clippy::match_same_arms)]
        match self {
            Undetermined => "Unknown",
            Eof => "EOF",
            Skip => "Skipped",
            HashbangComment => "#!",
            Ident => "Identifier",
            Await => "await",
            Break => "break",
            Case => "case",
            Catch => "catch",
            Class => "class",
            Const => "const",
            Continue => "continue",
            Debugger => "debugger",
            Default => "default",
            Delete => "delete",
            Do => "do",
            Else => "else",
            Enum => "enum",
            Export => "export",
            Extends => "extends",
            Finally => "finally",
            For => "for",
            Function => "function",
            Using => "using",
            If => "if",
            Import => "import",
            In => "in",
            Instanceof => "instanceof",
            New => "new",
            Return => "return",
            Super => "super",
            Switch => "switch",
            This => "this",
            Throw => "throw",
            Try => "try",
            Typeof => "typeof",
            Var => "var",
            Void => "void",
            While => "while",
            With => "with",
            As => "as",
            Async => "async",
            From => "from",
            Get => "get",
            Meta => "meta",
            Of => "of",
            Set => "set",
            Asserts => "asserts",
            Accessor => "accessor",
            Abstract => "abstract",
            Readonly => "readonly",
            Declare => "declare",
            Override => "override",
            Type => "type",
            Target => "target",
            Source => "source",
            Defer => "defer",
            Implements => "implements",
            Interface => "interface",
            Package => "package",
            Private => "private",
            Protected => "protected",
            Public => "public",
            Static => "static",
            Let => "let",
            Yield => "yield",
            Amp => "&",
            Amp2 => "&&",
            Amp2Eq => "&&=",
            AmpEq => "&=",
            Bang => "!",
            Caret => "^",
            CaretEq => "^=",
            Colon => ":",
            Comma => ",",
            Dot => ".",
            Dot3 => "...",
            Eq => "=",
            Eq2 => "==",
            Eq3 => "===",
            GtEq => ">=",
            LAngle => "<",
            LBrack => "[",
            LCurly => "{",
            LParen => "(",
            LtEq => "<=",
            Minus => "-",
            Minus2 => "--",
            MinusEq => "-=",
            Neq => "!=",
            Neq2 => "!==",
            Percent => "%",
            PercentEq => "%=",
            Pipe => "|",
            Pipe2 => "||",
            Pipe2Eq => "||=",
            PipeEq => "|=",
            Plus => "+",
            Plus2 => "++",
            PlusEq => "+=",
            Question => "?",
            Question2 => "??",
            Question2Eq => "??=",
            QuestionDot => "?.",
            RAngle => ">",
            RBrack => "]",
            RCurly => "}",
            RParen => ")",
            Semicolon => ";",
            ShiftLeft => "<<",
            ShiftLeftEq => "<<=",
            ShiftRight => ">>",
            ShiftRight3 => ">>>",
            ShiftRight3Eq => ">>>=",
            ShiftRightEq => ">>=",
            Slash => "/",
            SlashEq => "/=",
            Star => "*",
            Star2 => "**",
            Star2Eq => "**=",
            StarEq => "*=",
            Tilde => "~",
            Arrow => "=>",
            Null => "null",
            True => "true",
            False => "false",
            Decimal => "decimal",
            Float | PositiveExponential | NegativeExponential => "float",
            Binary => "binary",
            Octal => "octal",
            Hex => "hex",
            Str | String => "string",
            RegExp => "/regexp/",
            NoSubstitutionTemplate => "${}",
            TemplateHead => "${",
            TemplateMiddle => "${expr}",
            TemplateTail => "}",
            PrivateIdentifier => "#identifier",
            JSXText => "jsx",
            At => "@",
            Assert => "assert",
            Any => "any",
            Boolean => "boolean",
            Constructor => "constructor",
            Infer => "infer",
            Intrinsic => "intrinsic",
            Is => "is",
            KeyOf => "keyof",
            Module => "module",
            Namespace => "namaespace",
            Never => "never",
            Out => "out",
            Require => "require",
            Number => "number",
            Object => "object",
            Satisfies => "satisfies",
            Symbol => "symbol",
            Undefined => "undefined",
            Unique => "unique",
            Unknown => "unknown",
            Global => "global",
            BigInt => "bigint",
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_str().fmt(f)
    }
}
