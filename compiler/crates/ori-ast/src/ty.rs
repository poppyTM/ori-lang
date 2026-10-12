use crate::common::{Name, QualifiedName};
use crate::expr::{BinaryOp, Expr, UnaryOp};
use ori_diagnostics::Span;

/// The side-effect-free expression behind a named const type argument.
///
/// This deliberately mirrors only the expression forms that CT-0 can evaluate.
/// Keeping it separate from [`Expr`] prevents calls, allocation, I/O, and other
/// runtime operations from reaching type lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstExpr {
    Int {
        raw: smol_str::SmolStr,
        span: Span,
    },
    Bool(bool, Span),
    Name(QualifiedName),
    Unary {
        op: UnaryOp,
        operand: Box<Self>,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Self>,
        rhs: Box<Self>,
        span: Span,
    },
    If {
        condition: Box<Self>,
        then_expr: Box<Self>,
        else_expr: Box<Self>,
        span: Span,
    },
}

impl ConstExpr {
    /// Restrict a parsed runtime expression to the CT-0 expression subset.
    pub fn from_expr(expr: Expr) -> Result<Self, Span> {
        match expr {
            Expr::IntLit { raw, span } => Ok(Self::Int { raw, span }),
            Expr::BoolLit(value, span) => Ok(Self::Bool(value, span)),
            Expr::Ident(name) => Ok(Self::Name(QualifiedName::single(name))),
            Expr::QualifiedIdent(name) => Ok(Self::Name(name)),
            Expr::Unary { op, operand, span } => Ok(Self::Unary {
                op,
                operand: Box::new(Self::from_expr(*operand)?),
                span,
            }),
            Expr::Binary { op, lhs, rhs, span } => Ok(Self::Binary {
                op,
                lhs: Box::new(Self::from_expr(*lhs)?),
                rhs: Box::new(Self::from_expr(*rhs)?),
                span,
            }),
            Expr::IfExpr {
                condition,
                then_expr,
                else_expr,
                span,
            } => Ok(Self::If {
                condition: Box::new(Self::from_expr(*condition)?),
                then_expr: Box::new(Self::from_expr(*then_expr)?),
                else_expr: Box::new(Self::from_expr(*else_expr)?),
                span,
            }),
            other => Err(other.span()),
        }
    }

    pub const fn span(&self) -> Span {
        match self {
            Self::Int { span, .. }
            | Self::Bool(_, span)
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::If { span, .. } => *span,
            Self::Name(name) => name.span,
        }
    }
}

/// Every type that can appear in an Ori program.
///
/// Primitive types are explicit variants so the type checker can recognise them
/// without a symbol-table lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // ── Primitive types ──────────────────────────────────────────────────────
    Bool(Span),
    Int(Span),
    Int8(Span),
    Int16(Span),
    Int32(Span),
    Int64(Span),
    U8(Span),
    U16(Span),
    U32(Span),
    U64(Span),
    Float(Span),
    Float32(Span),
    Float64(Span),
    String(Span),
    Bytes(Span),
    Void(Span),

    // ── Named types ───────────────────────────────────────────────────────────
    /// A user-defined type by name: `User`, `app.config.Config`.
    Named(QualifiedName),

    /// A compile-time constant used as a type argument: `size: 8` in
    /// `Buffer[size: 8]`.
    ///
    /// Named on purpose. A bare number between brackets reads as an index
    /// everywhere else in Ori, so const arguments carry the parameter's name
    /// the same way call arguments and struct fields do.
    ConstArg {
        name: Name,
        value: ConstExpr,
        span: Span,
    },

    // ── Built-in generic types ────────────────────────────────────────────────
    Optional(Box<Self>, Span),
    Result(Box<Self>, Box<Self>, Span),
    List(Box<Self>, Span),
    /// `buffer[T]` — a mutable, contiguous, fixed-length sequence.
    Buffer(Box<Self>, Span),
    /// `slice[T]` — a read-only window over a `list[T]`.
    Slice(Box<Self>, Span),
    /// `array[T, size: N]` — fixed length, part of the type.
    ///
    /// Distinct from `list[T]`: the length is known at compile time, so the
    /// elements live inline instead of behind a heap pointer, and two arrays of
    /// different lengths are different types.
    Array {
        elem: Box<Self>,
        size: ConstExpr,
        span: Span,
    },
    /// `simd[T, lanes: N]` or `simd[T, N]` — portable fixed-width SIMD vector (LANG-SIMD-1).
    Simd {
        elem: Box<Self>,
        lanes: u16,
        span: Span,
    },
    Map(Box<Self>, Box<Self>, Span),
    Set(Box<Self>, Span),
    Range(Box<Self>, Span),
    Lazy(Box<Self>, Span),
    Handle(Box<Self>, Span),
    /// `any<Trait>` — dynamic dispatch.
    Any(QualifiedName, Span),
    /// `tuple<A, B, …>` — always at least 2 type arguments.
    Tuple(Vec<Self>, Span),

    // ── Callable types ────────────────────────────────────────────────────────
    /// `func(T, U) -> R`  or `func(T)` (void return → `None`).
    Func {
        params: Vec<Self>,
        return_ty: Option<Box<Self>>,
        span: Span,
    },

    // ── User-defined generic types ────────────────────────────────────────────
    /// `MyContainer<T>`, `Either<Left, Right>`.
    Generic {
        name: QualifiedName,
        args: Vec<Self>,
        span: Span,
    },
}

impl Type {
    pub const fn span(&self) -> Span {
        match self {
            Self::Bool(s)
            | Self::Int(s)
            | Self::Int8(s)
            | Self::Int16(s)
            | Self::Int32(s)
            | Self::Int64(s)
            | Self::U8(s)
            | Self::U16(s)
            | Self::U32(s)
            | Self::U64(s)
            | Self::Float(s)
            | Self::Float32(s)
            | Self::Float64(s)
            | Self::String(s)
            | Self::Bytes(s)
            | Self::Void(s) => *s,
            Self::Named(q) => q.span,
            Self::ConstArg { span, .. } => *span,
            Self::Buffer(_, s)
            | Self::Slice(_, s)
            | Self::Array { span: s, .. }
            | Self::Simd { span: s, .. }
            | Self::Optional(_, s)
            | Self::List(_, s)
            | Self::Set(_, s)
            | Self::Range(_, s)
            | Self::Lazy(_, s)
            | Self::Handle(_, s)
            | Self::Any(_, s)
            | Self::Tuple(_, s)
            | Self::Result(_, _, s)
            | Self::Map(_, _, s) => *s,
            Self::Func { span, .. } | Self::Generic { span, .. } => *span,
        }
    }
}
