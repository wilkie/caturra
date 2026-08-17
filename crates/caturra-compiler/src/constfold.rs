//! Compile-time constant expressions (JLS §15.29).
//!
//! A *constant expression* is more than a literal: `1 + 1`, `(char) 65`,
//! `true && false`, `"j" + "b"`, `Integer.MAX_VALUE` and a conditional over
//! constants are all constants, and so is any `final` variable initialized with
//! one — a *constant variable* (§4.12.4), whose reads javac INLINES.
//!
//! That distinction is not academic. Three separate rules key off it, and
//! caturra used to recognise only bare literals, so all three were wrong for a
//! constant whose initializer had an operator in it:
//!
//! * a `case` label must be a constant expression — `case MODE:` for a
//!   `static final int MODE = 1 + 1;` was refused outright;
//! * reading a constant variable does NOT initialize its class, because the
//!   value was inlined and the class is never touched — caturra ran the static
//!   initializer and printed what a JDK never prints;
//! * `while (Cfg.DEBUG)` over a `static final boolean DEBUG = false` makes the
//!   body unreachable, which javac reports as an error.
//!
//! Name resolution is left to the caller. The folder is used both while the
//! method table is being BUILT (where user constants are not yet known and only
//! library constants resolve) and from codegen (where a `final` local or a
//! `static final` field of any class does), so callers pass a `resolve` closure
//! rather than the folder reaching for a table it may not have.

use crate::ast::{BinaryOp, Expr, Literal, TypeRef, UnaryOp};

/// A folded constant, in the type Java gives it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConstValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Char(u16),
    Str(String),
}

impl ConstValue {
    /// The value as a `long`, for the integral types (`char` included, as its
    /// numeric promotion gives).
    fn integral(&self) -> Option<i64> {
        match self {
            ConstValue::Int(v) => Some(i64::from(*v)),
            ConstValue::Long(v) => Some(*v),
            ConstValue::Char(c) => Some(i64::from(*c)),
            _ => None,
        }
    }

    fn numeric(&self) -> Option<f64> {
        match self {
            ConstValue::Float(v) => Some(f64::from(*v)),
            ConstValue::Double(v) => Some(*v),
            other => other.integral().map(|v| {
                #[allow(clippy::cast_precision_loss)] // a long IS wider than a double
                let widened = v as f64;
                widened
            }),
        }
    }

    fn is_floating(&self) -> bool {
        matches!(self, ConstValue::Float(_) | ConstValue::Double(_))
    }

    /// How the value appears inside a folded string concatenation — the same
    /// text `String.valueOf` would give.
    ///
    /// `None` for a floating value: rendering one exactly as a JDK does is the
    /// job of the runtime formatter, so a concatenation involving one is left
    /// un-folded (it then takes the correct runtime path, and simply is not a
    /// constant expression here).
    pub(crate) fn java_string(&self) -> Option<String> {
        Some(match self {
            ConstValue::Int(v) => v.to_string(),
            ConstValue::Long(v) => v.to_string(),
            ConstValue::Bool(b) => b.to_string(),
            ConstValue::Char(c) => char::from_u32(u32::from(*c)).map(String::from)?,
            ConstValue::Str(s) => s.clone(),
            ConstValue::Float(_) | ConstValue::Double(_) => return None,
        })
    }

    /// The AST literal for this value, for the callers that store one.
    pub(crate) fn literal(&self) -> Literal {
        match self {
            ConstValue::Int(v) => Literal::Int(i64::from(*v)),
            ConstValue::Long(v) => Literal::Long(*v),
            ConstValue::Float(v) => Literal::Float(*v),
            ConstValue::Double(v) => Literal::Double(*v),
            ConstValue::Bool(b) => Literal::Bool(*b),
            ConstValue::Char(c) => Literal::Char(*c),
            ConstValue::Str(s) => Literal::Str(s.clone()),
        }
    }
}

/// Fold an expression to its compile-time constant value, or `None` when it is
/// not one. `resolve` answers for a NAME (`MODE`, `Cfg.DEBUG`,
/// `Integer.MAX_VALUE`) — a caller that knows no names may return `None` for
/// everything, which simply makes fewer expressions constant.
pub(crate) fn fold(
    expr: &Expr,
    resolve: &mut dyn FnMut(&[String]) -> Option<ConstValue>,
) -> Option<ConstValue> {
    match expr {
        Expr::Literal { value, .. } => literal_value(value),
        Expr::Name { path, .. } => resolve(path),
        Expr::Unary { op, operand, .. } => {
            let value = fold(operand, resolve)?;
            unary(*op, &value)
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            let l = fold(lhs, resolve)?;
            let r = fold(rhs, resolve)?;
            binary(*op, &l, &r)
        }
        Expr::Cast { ty, operand, .. } => {
            let value = fold(operand, resolve)?;
            cast(ty, &value)
        }
        // JLS §15.29: a conditional is constant when all three parts are. Only
        // the taken branch's VALUE matters, but the untaken one must still be
        // constant for the whole to be.
        Expr::Ternary {
            cond,
            then: then_expr,
            els: els_expr,
            ..
        } => {
            let ConstValue::Bool(taken) = fold(cond, resolve)? else {
                return None;
            };
            let (then, els) = (fold(then_expr, resolve)?, fold(els_expr, resolve)?);
            // The taken branch supplies the VALUE, but the conditional's own
            // TYPE is the promotion of both branches (JLS §15.25) and the value
            // converts to it. Returning the branch untouched made
            // `true ? 1 : 2.0` print `1` where Java prints `1.0`, and
            // `false ? 'a' : 98` print `98` where Java prints `b` — a folded
            // constant answering with the wrong type, which the emitter's own
            // conditional gets right when the condition is a variable.
            let value = if taken { &then } else { &els };
            // A `(byte)`/`(short)` cast makes the operand that TYPE, not an int
            // constant — so `true ? 'a' : (byte) 3` promotes to int (97) where
            // `true ? 'a' : 3` stays a char. `ConstValue` has no byte or short
            // of its own, so the distinction is read off the expression.
            let narrow = (is_narrow_cast(then_expr), is_narrow_cast(els_expr));
            Some(
                conditional_promotion(&then, &els, narrow)
                    .map_or_else(|| value.clone(), |to| to.apply(value)),
            )
        }
        _ => None,
    }
}

/// The conversion a constant conditional's branches undergo (JLS §15.25). A
/// non-numeric pair (two booleans, two strings) needs none, and a mixed one
/// is not a constant conditional at all.
fn conditional_promotion(
    then: &ConstValue,
    els: &ConstValue,
    narrow: (bool, bool),
) -> Option<Promotion> {
    use ConstValue::{Char, Double, Float, Long};
    let numeric = |v: &ConstValue| v.numeric().is_some();
    let fits_char = |v: &ConstValue, is_narrow: bool| {
        !is_narrow && matches!(v, ConstValue::Int(n) if u16::try_from(*n).is_ok())
    };
    if !numeric(then) || !numeric(els) {
        return None;
    }
    // A `char` beside an int CONSTANT that fits in one stays a char — the rule
    // that makes `flag ? 'a' : 98` a character rather than a number.
    let char_pair = matches!((then, els), (Char(_), Char(_)))
        || matches!(then, Char(_)) && fits_char(els, narrow.1)
        || matches!(els, Char(_)) && fits_char(then, narrow.0);
    Some(if char_pair {
        Promotion::Char
    } else if matches!(then, Double(_)) || matches!(els, Double(_)) {
        Promotion::Double
    } else if matches!(then, Float(_)) || matches!(els, Float(_)) {
        Promotion::Float
    } else if matches!(then, Long(_)) || matches!(els, Long(_)) {
        Promotion::Long
    } else {
        Promotion::Int
    })
}

/// Whether an expression is a cast to `byte` or `short` — an operand of that
/// TYPE rather than an int constant, which the conditional's own typing rules
/// distinguish.
fn is_narrow_cast(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Cast {
            ty: TypeRef::Byte | TypeRef::Short,
            ..
        }
    )
}

#[derive(Clone, Copy)]
enum Promotion {
    Char,
    Int,
    Long,
    Float,
    Double,
}

impl Promotion {
    fn apply(self, value: &ConstValue) -> ConstValue {
        match self {
            Self::Char => ConstValue::Char(
                value
                    .integral()
                    .and_then(|v| u16::try_from(v).ok())
                    .unwrap_or_default(),
            ),
            Self::Int => ConstValue::Int(
                value
                    .integral()
                    .and_then(|v| i32::try_from(v).ok())
                    .unwrap_or_default(),
            ),
            Self::Long => ConstValue::Long(value.integral().unwrap_or_default()),
            #[allow(clippy::cast_possible_truncation)] // a float IS narrower
            Self::Float => ConstValue::Float(value.numeric().unwrap_or_default() as f32),
            Self::Double => ConstValue::Double(value.numeric().unwrap_or_default()),
        }
    }
}

/// A declared constant's literal as a foldable value — what a caller holding
/// a class's constant fields hands to [`fold`]'s resolver.
pub(crate) fn literal_const(value: &Literal) -> Option<ConstValue> {
    literal_value(value)
}

fn literal_value(value: &Literal) -> Option<ConstValue> {
    Some(match value {
        Literal::Int(v) => ConstValue::Int(i32::try_from(*v).ok()?),
        Literal::Long(v) => ConstValue::Long(*v),
        Literal::Float(v) => ConstValue::Float(*v),
        Literal::Double(v) => ConstValue::Double(*v),
        Literal::Bool(b) => ConstValue::Bool(*b),
        Literal::Char(c) => ConstValue::Char(u16::try_from(u32::from(*c)).ok()?),
        Literal::Str(s) => ConstValue::Str(s.clone()),
        Literal::Null => return None,
    })
}

/// The `java.lang` constants a program may name in a constant expression.
/// Nothing else is a constant VARIABLE in the library: `Math.PI` is one too,
/// and is included, but a method call never is.
pub(crate) fn library_constant(path: &[String]) -> Option<ConstValue> {
    let [class, name] = path else {
        return None;
    };
    let class = class.rsplit('.').next().unwrap_or(class);
    Some(match (class, name.as_str()) {
        ("Integer", "MAX_VALUE") => ConstValue::Int(i32::MAX),
        ("Integer", "MIN_VALUE") => ConstValue::Int(i32::MIN),
        ("Long", "MAX_VALUE") => ConstValue::Long(i64::MAX),
        ("Long", "MIN_VALUE") => ConstValue::Long(i64::MIN),
        ("Short", "MAX_VALUE") => ConstValue::Int(i32::from(i16::MAX)),
        ("Short", "MIN_VALUE") => ConstValue::Int(i32::from(i16::MIN)),
        ("Byte", "MAX_VALUE") => ConstValue::Int(i32::from(i8::MAX)),
        ("Byte", "MIN_VALUE") => ConstValue::Int(i32::from(i8::MIN)),
        ("Character", "MAX_VALUE") => ConstValue::Char(u16::MAX),
        ("Character", "MIN_VALUE") => ConstValue::Char(0),
        ("Double", "MAX_VALUE") => ConstValue::Double(f64::MAX),
        ("Double", "MIN_VALUE") => ConstValue::Double(f64::MIN_POSITIVE * f64::EPSILON / 2.0),
        ("Float", "MAX_VALUE") => ConstValue::Float(f32::MAX),
        ("Math", "PI") => ConstValue::Double(std::f64::consts::PI),
        ("Math", "E") => ConstValue::Double(std::f64::consts::E),
        _ => return None,
    })
}

fn unary(op: UnaryOp, value: &ConstValue) -> Option<ConstValue> {
    match op {
        UnaryOp::Not => match value {
            ConstValue::Bool(b) => Some(ConstValue::Bool(!b)),
            _ => None,
        },
        // Unary minus and plus promote (§5.6.1): a `char`/`short`/`byte`
        // operand becomes an `int`, so `-'a'` is an int.
        UnaryOp::Neg | UnaryOp::Plus => {
            let negate = op == UnaryOp::Neg;
            Some(match promote_unary(value)? {
                ConstValue::Int(v) => ConstValue::Int(if negate { v.wrapping_neg() } else { v }),
                ConstValue::Long(v) => ConstValue::Long(if negate { v.wrapping_neg() } else { v }),
                ConstValue::Float(v) => ConstValue::Float(if negate { -v } else { v }),
                ConstValue::Double(v) => ConstValue::Double(if negate { -v } else { v }),
                _ => return None,
            })
        }
        UnaryOp::BitNot => Some(match promote_unary(value)? {
            ConstValue::Int(v) => ConstValue::Int(!v),
            ConstValue::Long(v) => ConstValue::Long(!v),
            _ => return None,
        }),
    }
}

fn promote_unary(value: &ConstValue) -> Option<ConstValue> {
    Some(match value {
        ConstValue::Char(c) => ConstValue::Int(i32::from(*c)),
        ConstValue::Int(v) => ConstValue::Int(*v),
        ConstValue::Long(v) => ConstValue::Long(*v),
        ConstValue::Float(v) => ConstValue::Float(*v),
        ConstValue::Double(v) => ConstValue::Double(*v),
        _ => return None,
    })
}

#[allow(clippy::too_many_lines)] // one arm per operator, as the JLS lists them
fn binary(op: BinaryOp, l: &ConstValue, r: &ConstValue) -> Option<ConstValue> {
    // String concatenation: a `+` with a String operand, where the other side
    // is any constant at all.
    if op == BinaryOp::Add && matches!((l, r), (ConstValue::Str(_), _) | (_, ConstValue::Str(_))) {
        return Some(ConstValue::Str(format!(
            "{}{}",
            l.java_string()?,
            r.java_string()?
        )));
    }
    match op {
        BinaryOp::And => {
            return match (l, r) {
                (ConstValue::Bool(a), ConstValue::Bool(b)) => Some(ConstValue::Bool(*a && *b)),
                _ => None,
            };
        }
        BinaryOp::Or => {
            return match (l, r) {
                (ConstValue::Bool(a), ConstValue::Bool(b)) => Some(ConstValue::Bool(*a || *b)),
                _ => None,
            };
        }
        BinaryOp::Eq | BinaryOp::Ne => {
            let same = match (l, r) {
                (ConstValue::Bool(a), ConstValue::Bool(b)) => a == b,
                // Two String CONSTANTS are the same interned object, so `==`
                // over them is a constant comparison of their text.
                (ConstValue::Str(a), ConstValue::Str(b)) => a == b,
                _ => {
                    let (a, b) = (l.numeric()?, r.numeric()?);
                    #[allow(clippy::float_cmp)] // Java's `==` on floats IS exact
                    let equal = a == b;
                    equal
                }
            };
            return Some(ConstValue::Bool(if op == BinaryOp::Eq {
                same
            } else {
                !same
            }));
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            let (a, b) = (l.numeric()?, r.numeric()?);
            return Some(ConstValue::Bool(match op {
                BinaryOp::Lt => a < b,
                BinaryOp::Le => a <= b,
                BinaryOp::Gt => a > b,
                _ => a >= b,
            }));
        }
        // A SHIFT does not promote its operands together: the result's type is
        // the left operand's, and the right is masked to 5 or 6 bits (§15.19).
        BinaryOp::Shl | BinaryOp::Shr | BinaryOp::Ushr => {
            let distance = r.integral()?;
            return match promote_unary(l)? {
                ConstValue::Int(v) => {
                    #[allow(clippy::cast_sign_loss)] // masked to 0..=31 first
                    let s = (distance & 31) as u32;
                    Some(ConstValue::Int(match op {
                        BinaryOp::Shl => v.wrapping_shl(s),
                        BinaryOp::Shr => v.wrapping_shr(s),
                        _ => (v.cast_unsigned().wrapping_shr(s)).cast_signed(),
                    }))
                }
                ConstValue::Long(v) => {
                    #[allow(clippy::cast_sign_loss)] // masked to 0..=63 first
                    let s = (distance & 63) as u32;
                    Some(ConstValue::Long(match op {
                        BinaryOp::Shl => v.wrapping_shl(s),
                        BinaryOp::Shr => v.wrapping_shr(s),
                        _ => (v.cast_unsigned().wrapping_shr(s)).cast_signed(),
                    }))
                }
                _ => None,
            };
        }
        _ => {}
    }
    // Bitwise operators over two booleans are boolean (`&`, `|`, `^`).
    if let (ConstValue::Bool(a), ConstValue::Bool(b)) = (l, r) {
        return match op {
            BinaryOp::BitAnd => Some(ConstValue::Bool(*a && *b)),
            BinaryOp::BitOr => Some(ConstValue::Bool(*a || *b)),
            BinaryOp::BitXor => Some(ConstValue::Bool(a != b)),
            _ => None,
        };
    }
    // Binary numeric promotion (§5.6.2), then the operator in that type.
    if l.is_floating() || r.is_floating() {
        let double = matches!(l, ConstValue::Double(_)) || matches!(r, ConstValue::Double(_));
        let (a, b) = (l.numeric()?, r.numeric()?);
        let value = match op {
            BinaryOp::Add => a + b,
            BinaryOp::Sub => a - b,
            BinaryOp::Mul => a * b,
            BinaryOp::Div => a / b,
            BinaryOp::Rem => a % b,
            _ => return None,
        };
        #[allow(clippy::cast_possible_truncation)] // the float case IS a narrowing
        return Some(if double {
            ConstValue::Double(value)
        } else {
            ConstValue::Float(value as f32)
        });
    }
    let long = matches!(l, ConstValue::Long(_)) || matches!(r, ConstValue::Long(_));
    let (a, b) = (l.integral()?, r.integral()?);
    if long {
        integral(op, a, b).map(ConstValue::Long)
    } else {
        #[allow(clippy::cast_possible_truncation)]
        let (a, b) = (a as i32, b as i32);
        integral_int(op, a, b).map(ConstValue::Int)
    }
}

fn integral(op: BinaryOp, a: i64, b: i64) -> Option<i64> {
    Some(match op {
        BinaryOp::Add => a.wrapping_add(b),
        BinaryOp::Sub => a.wrapping_sub(b),
        BinaryOp::Mul => a.wrapping_mul(b),
        // A constant division by zero is not a constant expression here: it
        // takes the runtime path and throws ArithmeticException, rather than
        // being folded into a value that does not exist.
        BinaryOp::Div => a.checked_div(b)?,
        BinaryOp::Rem => a.checked_rem(b)?,
        BinaryOp::BitAnd => a & b,
        BinaryOp::BitOr => a | b,
        BinaryOp::BitXor => a ^ b,
        _ => return None,
    })
}

fn integral_int(op: BinaryOp, a: i32, b: i32) -> Option<i32> {
    Some(match op {
        BinaryOp::Add => a.wrapping_add(b),
        BinaryOp::Sub => a.wrapping_sub(b),
        BinaryOp::Mul => a.wrapping_mul(b),
        BinaryOp::Div => a.checked_div(b)?,
        BinaryOp::Rem => a.checked_rem(b)?,
        BinaryOp::BitAnd => a & b,
        BinaryOp::BitOr => a | b,
        BinaryOp::BitXor => a ^ b,
        _ => return None,
    })
}

/// A cast inside a constant expression (§5.5): to a primitive type or to
/// `String`, which is the only reference cast a constant survives.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)] // every one of these IS a Java narrowing conversion
fn cast(ty: &TypeRef, value: &ConstValue) -> Option<ConstValue> {
    let name = match ty {
        TypeRef::Int => "int",
        TypeRef::Long => "long",
        TypeRef::Double => "double",
        TypeRef::Float => "float",
        TypeRef::Short => "short",
        TypeRef::Byte => "byte",
        TypeRef::Char => "char",
        TypeRef::Boolean => "boolean",
        TypeRef::Named(name) => match name.rsplit('.').next().unwrap_or(name) {
            "String" => "String",
            _ => return None,
        },
        _ => return None,
    };
    Some(match name {
        "boolean" => match value {
            ConstValue::Bool(b) => ConstValue::Bool(*b),
            _ => return None,
        },
        "String" => match value {
            ConstValue::Str(s) => ConstValue::Str(s.clone()),
            _ => return None,
        },
        "double" => ConstValue::Double(value.numeric()?),
        "float" => ConstValue::Float(value.numeric()? as f32),
        "long" => ConstValue::Long(to_long(value)?),
        "int" => ConstValue::Int(to_int(value)?),
        // byte/short/char narrow from the INT the value converts to first
        // (JLS §5.1.3), so `(byte) 1e10` is `(byte) Integer.MAX_VALUE`.
        "short" => ConstValue::Int(i32::from(to_int(value)? as i16)),
        "byte" => ConstValue::Int(i32::from(to_int(value)? as i8)),
        _ => ConstValue::Char(to_int(value)? as u16),
    })
}

/// A narrowing conversion to an integral type goes through `long`, and from a
/// floating value it truncates toward zero and saturates (§5.1.3).
#[allow(clippy::cast_possible_truncation)]
fn to_long(value: &ConstValue) -> Option<i64> {
    match value {
        ConstValue::Float(v) => Some(*v as i64),
        ConstValue::Double(v) => Some(*v as i64),
        other => other.integral(),
    }
}

/// The `int` a value converts to. A FLOATING source saturates straight into
/// int range (§5.1.3) — it does not pass through `long` first, which is what
/// made `(int) (1.0 / 0.0)` fold to -1 (`long`'s saturated maximum, truncated)
/// where Java, and caturra's own runtime, answer `Integer.MAX_VALUE`.
#[allow(clippy::cast_possible_truncation)]
fn to_int(value: &ConstValue) -> Option<i32> {
    match value {
        ConstValue::Float(v) => Some(*v as i32),
        ConstValue::Double(v) => Some(*v as i32),
        other => Some(other.integral()? as i32),
    }
}
