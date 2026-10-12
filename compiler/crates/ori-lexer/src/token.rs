use logos::{Lexer, Logos};

// ── callbacks ────────────────────────────────────────────────────────────────

/// Consumes everything up to and including the closing `|--`.
/// Called after `--|` has already been consumed by logos.
fn lex_block_comment(lex: &mut Lexer<TokenKind>) -> bool {
    match lex.remainder().find("|--") {
        Some(end) => {
            lex.bump(end + 3);
            true
        }
        None => false,
    }
}

/// Consumes everything up to and including the closing `"""`.
/// Called after the opening `"""` (or `f"""`) has been consumed.
fn lex_triple_str_body(lex: &mut Lexer<TokenKind>) -> bool {
    match lex.remainder().find("\"\"\"") {
        Some(end) => {
            lex.bump(end + 3);
            true
        }
        None => false,
    }
}

// ── token kinds ──────────────────────────────────────────────────────────────

/// Every token the Ori lexer can produce.
///
/// Notes:
/// - Whitespace and line-comments are **skipped** (not emitted).
/// - Block/doc comments (`--| … |--`) are emitted as `BlockComment` so that
///   `ori doc` can process them; the main compiler can ignore them.
/// - Keywords are matched before `Ident` due to logos' token-priority rules.
/// - Primitive type names (`bool`, `int`, …) are reserved keywords.
#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\r\n]+")] // skip whitespace
pub enum TokenKind {
    // ── Comments ─────────────────────────────────────────────────────────────
    /// A line comment: `-- text`.
    /// The `--|` case is consumed by `BlockComment` first (token priority).
    #[regex(r"--[^\n]*")]
    LineComment,

    /// A block / doc comment: `--| … |--`.
    #[regex(r"--\|", lex_block_comment)]
    BlockComment,

    // ── Reserved keywords ────────────────────────────────────────────────────
    /// Canonical file header keyword (S3 surface).
    #[token("module")]
    Module,
    /// Removed in S3; kept so the parser can emit a hard error on the old form.
    #[token("namespace")]
    Namespace,
    #[token("import")]
    Import,
    #[token("as")]
    As,
    #[token("public")]
    Public,
    /// Still used for callable types `func(T) -> R`. Declaration form no longer
    /// uses this keyword (S3); the parser errors when it appears on declarations.
    #[token("func")]
    Func,
    #[token("return")]
    Return,
    #[token("end")]
    End,
    #[token("const")]
    Const,
    #[token("var")]
    Var,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    /// Canonical chained-conditional keyword (S3 surface). Replaces `else if`.
    #[token("elif")]
    Elif,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("repeat")]
    Repeat,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("match")]
    Match,
    #[token("case")]
    Case,
    #[token("loop")]
    Loop,
    #[token("struct")]
    Struct,
    #[token("trait")]
    Trait,
    /// S3 surface: `apply Type … end` with `use Trait` sections.
    #[token("apply")]
    Apply,
    /// S3 surface: `use Trait … end` inside an `apply` block.
    #[token("use")]
    Use,
    /// Removed in S3; kept so the parser can emit a hard error on the old form.
    #[token("implement")]
    Implement,
    #[token("enum")]
    Enum,
    #[token("where")]
    Where,
    #[token("is")]
    Is,
    #[token("alias")]
    Alias,
    /// `newtype Name = Repr` — a distinct type with the same representation.
    #[token("newtype")]
    Newtype,
    #[token("do")]
    Do,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("none")]
    None,
    /// Removed result ctor (`success`); emits `parse.result_ctor_renamed`.
    #[token("success")]
    SuccessRemoved,
    /// Removed result ctor (`error`); emits `parse.result_ctor_renamed`.
    #[token("error")]
    ErrorRemoved,
    // `ok` / `err` are soft: parsed as Ident; special-cased when followed by `(`.
    #[token("some")]
    Some,
    #[token("mut")]
    Mut,
    #[token("self")]
    SelfKw,
    #[token("attr")]
    Attr,
    #[token("extern")]
    Extern,
    #[token("any")]
    Any,
    #[token("optional")]
    Optional,
    #[token("result")]
    ResultKw,
    #[token("list")]
    List,
    #[token("map")]
    Map,
    #[token("set")]
    Set,
    #[token("range")]
    Range,
    #[token("void")]
    Void,

    // Keywords used by statement, expression, and type grammar.
    #[token("using")]
    Using,
    #[token("check")]
    Check,
    #[token("with")]
    With,
    #[token("then")]
    Then,
    #[token("tuple")]
    Tuple,
    #[token("lazy")]
    Lazy,
    #[token("handle")]
    Handle,

    // ── Primitive type names (also reserved) ─────────────────────────────────
    #[token("bool")]
    BoolTy,
    #[token("int")]
    IntTy,
    #[token("int8")]
    Int8Ty,
    #[token("int16")]
    Int16Ty,
    #[token("int32")]
    Int32Ty,
    #[token("int64")]
    Int64Ty,
    #[token("u8")]
    U8Ty,
    #[token("u16")]
    U16Ty,
    #[token("u32")]
    U32Ty,
    #[token("u64")]
    U64Ty,
    #[token("float")]
    FloatTy,
    #[token("float32")]
    Float32Ty,
    #[token("float64")]
    Float64Ty,
    #[token("string")]
    StringTy,
    #[token("bytes")]
    BytesTy,

    // ── Identifiers ──────────────────────────────────────────────────────────
    /// A user-defined name: Unicode letter or `_`, followed by letters,
    /// decimal digits, or `_`.
    /// Keywords above have higher priority and are matched first.
    #[regex(r"[_\p{L}][_\p{L}\p{Nd}]*")]
    Ident,

    // ── Integer literals ─────────────────────────────────────────────────────
    /// Hex: `0xFF`, `0xDEAD_u32`
    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*(i(8|16|32|64)|u(8|16|32|64))?")]
    /// Binary: `0b1010`, `0b1111_u8`
    #[regex(r"0[bB][01][01_]*(i(8|16|32|64)|u(8|16|32|64))?")]
    /// Octal: `0o755`
    #[regex(r"0[oO][0-7][0-7_]*(i(8|16|32|64)|u(8|16|32|64))?")]
    /// Decimal: `42`, `1_000_000`, `42u8`, `100i32`
    #[regex(r"[0-9][0-9_]*(i(8|16|32|64)|u(8|16|32|64))?")]
    IntLit,

    // ── Float literals ───────────────────────────────────────────────────────
    /// `3.14`, `1.0e-5`, `6.022e23f32`
    /// Must have digits on both sides of `.` to avoid conflict with `..`.
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+\-]?[0-9][0-9_]*)?(f(32|64))?")]
    FloatLit,

    // ── String literals ──────────────────────────────────────────────────────
    /// Triple-quoted f-string: `f"""…"""`
    #[regex(r#"f""""#, lex_triple_str_body)]
    TripleFStrLit,

    /// Triple-quoted string: `"""…"""`
    #[regex(r#"""""#, lex_triple_str_body)]
    TripleStrLit,

    /// Interpolated string: `f"hello {name}"`
    /// Interpolation expressions are NOT parsed at the lexer level;
    /// the parser handles `{…}` inside f-string tokens.
    #[regex(r#"f"([^"\\]|\\.)*""#)]
    FStrLit,

    /// Plain string: `"hello\n"`
    #[regex(r#""([^"\\]|\\.)*""#)]
    StrLit,

    /// Byte string: `b"\xFF\x00"`
    #[regex(r#"b"([^"\\]|\\.)*""#)]
    BytesLit,

    // ── Operators (longer tokens declared first for priority) ────────────────
    #[token("-->")]
    Uninhabited, // intentionally unused; prevents `->` + `-` ambiguity notes
    #[token("...")]
    Ellipsis,
    #[token("..")]
    DotDot,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("|>")]
    Pipe,
    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token(".")]
    Dot,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("&")]
    Amp,
    #[token("|")]
    Bar,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("=")]
    Eq,
    #[token("?")]
    Question,

    // ── Delimiters ───────────────────────────────────────────────────────────
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("@")]
    At,
}

impl TokenKind {
    /// Returns `true` for trivia tokens that most passes can ignore.
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::LineComment | Self::BlockComment)
    }

    /// Human-readable name used in diagnostics.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Ident => "identifier",
            Self::IntLit => "integer literal",
            Self::FloatLit => "float literal",
            Self::StrLit | Self::FStrLit | Self::TripleStrLit | Self::TripleFStrLit => {
                "string literal"
            }
            Self::BytesLit => "bytes literal",
            Self::Module => "`module`",
            Self::Namespace => "`namespace`",
            Self::Import => "`import`",
            Self::As => "`as`",
            Self::Public => "`public`",
            Self::Func => "`func`",
            Self::Return => "`return`",
            Self::End => "`end`",
            Self::Const => "`const`",
            Self::Var => "`var`",
            Self::If => "`if`",
            Self::Else => "`else`",
            Self::Elif => "`elif`",
            Self::While => "`while`",
            Self::For => "`for`",
            Self::In => "`in`",
            Self::Repeat => "`repeat`",
            Self::Break => "`break`",
            Self::Continue => "`continue`",
            Self::Match => "`match`",
            Self::Case => "`case`",
            Self::Loop => "`loop`",
            Self::Struct => "`struct`",
            Self::Trait => "`trait`",
            Self::Apply => "`apply`",
            Self::Use => "`use`",
            Self::Implement => "`implement`",
            Self::Enum => "`enum`",
            Self::Where => "`where`",
            Self::Is => "`is`",
            Self::Alias => "`alias`",
            Self::Newtype => "`newtype`",
            Self::Do => "`do`",
            Self::And => "`and`",
            Self::Or => "`or`",
            Self::Not => "`not`",
            Self::True => "`true`",
            Self::False => "`false`",
            Self::None => "`none`",
            Self::SuccessRemoved => "`success`",
            Self::ErrorRemoved => "`error`",
            Self::Some => "`some`",
            Self::Mut => "`mut`",
            Self::SelfKw => "`self`",
            Self::Attr => "`attr`",
            Self::Extern => "`extern`",
            Self::Any => "`any`",
            Self::Optional => "`optional`",
            Self::ResultKw => "`result`",
            Self::List => "`list`",
            Self::Map => "`map`",
            Self::Set => "`set`",
            Self::Range => "`range`",
            Self::Void => "`void`",
            Self::Using => "`using`",
            Self::Check => "`check`",
            Self::With => "`with`",
            Self::Then => "`then`",
            Self::Tuple => "`tuple`",
            Self::Lazy => "`lazy`",
            Self::Handle => "`handle`",
            Self::BoolTy => "`bool`",
            Self::IntTy => "`int`",
            Self::Int8Ty => "`int8`",
            Self::Int16Ty => "`int16`",
            Self::Int32Ty => "`int32`",
            Self::Int64Ty => "`int64`",
            Self::U8Ty => "`u8`",
            Self::U16Ty => "`u16`",
            Self::U32Ty => "`u32`",
            Self::U64Ty => "`u64`",
            Self::FloatTy => "`float`",
            Self::Float32Ty => "`float32`",
            Self::Float64Ty => "`float64`",
            Self::StringTy => "`string`",
            Self::BytesTy => "`bytes`",
            Self::Ellipsis => "`...`",
            Self::DotDot => "`..`",
            Self::Arrow => "`->`",
            Self::FatArrow => "`=>`",
            Self::Pipe => "`|>`",
            Self::EqEq => "`==`",
            Self::BangEq => "`!=`",
            Self::LtEq => "`<=`",
            Self::GtEq => "`>=`",
            Self::PlusEq => "`+=`",
            Self::MinusEq => "`-=`",
            Self::StarEq => "`*=`",
            Self::SlashEq => "`/=`",
            Self::Shl => "`<<`",
            Self::Shr => "`>>`",
            Self::Dot => "`.`",
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Star => "`*`",
            Self::Slash => "`/`",
            Self::Percent => "`%`",
            Self::Amp => "`&`",
            Self::Bar => "`|`",
            Self::Caret => "`^`",
            Self::Tilde => "`~`",
            Self::Lt => "`<`",
            Self::Gt => "`>`",
            Self::Eq => "`=`",
            Self::Question => "`?`",
            Self::LParen => "`(`",
            Self::RParen => "`)`",
            Self::LBracket => "`[`",
            Self::RBracket => "`]`",
            Self::LBrace => "`{`",
            Self::RBrace => "`}`",
            Self::Colon => "`:`",
            Self::Comma => "`,`",
            Self::At => "`@`",
            Self::LineComment => "line comment",
            Self::BlockComment => "block comment",
            // `-->` is lexed as one token so parser diagnostics can explain
            // the exact invalid spelling instead of panicking while rendering
            // the token name.
            Self::Uninhabited => "`-->`",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TokenKind;

    #[test]
    fn every_token_kind_has_a_stable_display_name() {
        let all_tokens = [
            TokenKind::LineComment,
            TokenKind::BlockComment,
            TokenKind::Module,
            TokenKind::Namespace,
            TokenKind::Import,
            TokenKind::As,
            TokenKind::Public,
            TokenKind::Func,
            TokenKind::Return,
            TokenKind::End,
            TokenKind::Const,
            TokenKind::Var,
            TokenKind::If,
            TokenKind::Else,
            TokenKind::Elif,
            TokenKind::While,
            TokenKind::For,
            TokenKind::In,
            TokenKind::Repeat,
            TokenKind::Break,
            TokenKind::Continue,
            TokenKind::Match,
            TokenKind::Case,
            TokenKind::Loop,
            TokenKind::Struct,
            TokenKind::Trait,
            TokenKind::Apply,
            TokenKind::Use,
            TokenKind::Implement,
            TokenKind::Enum,
            TokenKind::Where,
            TokenKind::Is,
            TokenKind::Alias,
            TokenKind::Newtype,
            TokenKind::Do,
            TokenKind::And,
            TokenKind::Or,
            TokenKind::Not,
            TokenKind::True,
            TokenKind::False,
            TokenKind::None,
            TokenKind::SuccessRemoved,
            TokenKind::ErrorRemoved,
            TokenKind::Some,
            TokenKind::Mut,
            TokenKind::SelfKw,
            TokenKind::Attr,
            TokenKind::Extern,
            TokenKind::Any,
            TokenKind::Optional,
            TokenKind::ResultKw,
            TokenKind::List,
            TokenKind::Map,
            TokenKind::Set,
            TokenKind::Range,
            TokenKind::Void,
            TokenKind::Using,
            TokenKind::Check,
            TokenKind::With,
            TokenKind::Then,
            TokenKind::Tuple,
            TokenKind::Lazy,
            TokenKind::Handle,
            TokenKind::BoolTy,
            TokenKind::IntTy,
            TokenKind::Int8Ty,
            TokenKind::Int16Ty,
            TokenKind::Int32Ty,
            TokenKind::Int64Ty,
            TokenKind::U8Ty,
            TokenKind::U16Ty,
            TokenKind::U32Ty,
            TokenKind::U64Ty,
            TokenKind::FloatTy,
            TokenKind::Float32Ty,
            TokenKind::Float64Ty,
            TokenKind::StringTy,
            TokenKind::BytesTy,
            TokenKind::Ident,
            TokenKind::IntLit,
            TokenKind::FloatLit,
            TokenKind::TripleFStrLit,
            TokenKind::TripleStrLit,
            TokenKind::FStrLit,
            TokenKind::StrLit,
            TokenKind::BytesLit,
            TokenKind::Uninhabited,
            TokenKind::Ellipsis,
            TokenKind::DotDot,
            TokenKind::Arrow,
            TokenKind::FatArrow,
            TokenKind::Pipe,
            TokenKind::EqEq,
            TokenKind::BangEq,
            TokenKind::LtEq,
            TokenKind::GtEq,
            TokenKind::PlusEq,
            TokenKind::MinusEq,
            TokenKind::StarEq,
            TokenKind::SlashEq,
            TokenKind::Shl,
            TokenKind::Shr,
            TokenKind::Dot,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Amp,
            TokenKind::Bar,
            TokenKind::Caret,
            TokenKind::Tilde,
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::Eq,
            TokenKind::Question,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Colon,
            TokenKind::Comma,
            TokenKind::At,
        ];

        for token in all_tokens {
            assert!(
                !token.display_name().is_empty(),
                "missing display name for {token:?}"
            );
        }
    }
}
