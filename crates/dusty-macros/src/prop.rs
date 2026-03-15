use syn::parse::{Parse, ParseStream};
use syn::{Expr, Token};

/// Parsed `#[prop(...)]` modifiers for a single component parameter.
#[derive(Debug, Default)]
pub struct PropAttr {
    /// Use `impl Into<T>` for the parameter.
    pub into: bool,
    /// The parameter has a default value.
    pub default: Option<DefaultValue>,
    /// The parameter is optional (`Option<T>`, defaults to `None`).
    pub optional: bool,
}

/// The kind of default value on a `#[prop(default)]` or `#[prop(default = expr)]`.
#[derive(Debug)]
pub enum DefaultValue {
    /// `#[prop(default)]` — uses `Default::default()`.
    Trait,
    /// `#[prop(default = expr)]` — uses the given expression.
    Expr(Expr),
}

impl PropAttr {
    /// Whether this prop is required (appears in `new()`).
    pub const fn is_required(&self) -> bool {
        !self.optional && self.default.is_none()
    }
}

/// Parses the contents inside `#[prop(...)]`.
///
/// Grammar: comma-separated modifiers:
///   - `into`
///   - `default`
///   - `default = <expr>`
///   - `optional`
impl Parse for PropAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attr = Self::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "into" => {
                    attr.into = true;
                }
                "default" => {
                    if input.peek(Token![=]) {
                        let _: Token![=] = input.parse()?;
                        let expr: Expr = input.parse()?;
                        attr.default = Some(DefaultValue::Expr(expr));
                    } else {
                        attr.default = Some(DefaultValue::Trait);
                    }
                }
                "optional" => {
                    attr.optional = true;
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown prop modifier `{other}`, expected `into`, `default`, or `optional`"),
                    ));
                }
            }

            if !input.is_empty() {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(attr)
    }
}
