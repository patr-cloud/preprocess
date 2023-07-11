use syn::{
	spanned::Spanned,
	Error,
	Expr,
	ExprLit,
	Lit,
	LitFloat,
	LitInt,
	LitStr,
};

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
	fn require_int(self) -> Result<LitInt, Error>;
	fn require_float(self) -> Result<LitFloat, Error>;
}

impl LitExpr for Lit {
	fn require_str(self) -> Result<LitStr, Error> {
		match self {
			Lit::Str(lit) => Ok(lit),
			_ => Err(Error::new(self.span(), "expected string literal")),
		}
	}

	fn require_int(self) -> Result<LitInt, Error> {
		match self {
			Lit::Int(lit) => Ok(lit),
			_ => Err(Error::new(self.span(), "expected string literal")),
		}
	}

	fn require_float(self) -> Result<LitFloat, Error> {
		match self {
			Lit::Float(lit) => Ok(lit),
			_ => Err(Error::new(self.span(), "expected string literal")),
		}
	}
}
