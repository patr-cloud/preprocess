use syn::{Attribute, Error, Expr, ExprLit, Lit, LitStr, spanned::Spanned};

/// Returns the subset of `attrs` that affect whether a field exists at compile
/// time (`#[cfg(...)]`). These need to be echoed onto destructure patterns and
/// per-field `let`-statements so the generated `preprocess()` body matches the
/// cfg-resolved shape of the type.
///
/// `#[cfg_attr]` is intentionally excluded — its inner attribute could be
/// anything (e.g. `#[serde(...)]`), and emitting that into a pattern or
/// statement position would be a compile error.
pub fn cfg_attrs(attrs: &[Attribute]) -> Vec<&Attribute> {
	attrs
		.iter()
		.filter(|attr| attr.path().is_ident("cfg"))
		.collect()
}

pub trait ExprExt
where
	Self: Sized,
{
	fn require_lit(self) -> Result<ExprLit, Error>;
}

impl ExprExt for Expr {
	fn require_lit(self) -> Result<ExprLit, Error> {
		match self {
			Expr::Lit(lit) => Ok(lit),
			_ => Err(Error::new(self.span(), "expected literal")),
		}
	}
}

pub trait LitExpr {
	fn require_str(self) -> Result<LitStr, Error>;
}

impl LitExpr for Lit {
	fn require_str(self) -> Result<LitStr, Error> {
		match self {
			Lit::Str(lit) => Ok(lit),
			_ => Err(Error::new(self.span(), "expected string literal")),
		}
	}
}
