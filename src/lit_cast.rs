use syn::Lit;

#[derive(Debug, PartialEq)]
pub struct CastError;

pub trait FromLit: Sized {
    fn from_lit(lit: Lit) -> Result<Self, CastError>;
}

impl FromLit for Lit {
    fn from_lit(lit: Lit) -> Result<Self, CastError> {
        Ok(lit)
    }
}

impl FromLit for u64 {
    fn from_lit(lit: Lit) -> Result<Self, CastError> {
        match lit {
            Lit::Int(int) => int.base10_parse().map_err(|_| CastError),
            _ => Err(CastError),
        }
    }
}

impl FromLit for f64 {
    fn from_lit(lit: Lit) -> Result<Self, CastError> {
        match lit {
            Lit::Float(float) => float.base10_parse().map_err(|_| CastError),
            _ => Err(CastError),
        }
    }
}

impl FromLit for bool {
    fn from_lit(lit: Lit) -> Result<Self, CastError> {
        match lit {
            Lit::Bool(lit) => Ok(lit.value),
            _ => Err(CastError),
        }
    }
}

impl FromLit for String {
    fn from_lit(lit: Lit) -> Result<Self, CastError> {
        match lit {
            Lit::Str(string) => Ok(string.value()),
            _ => Err(CastError),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::{parse_quote, Lit};

    #[test]
    fn test_int() {
        let int_lit: Lit = parse_quote!(12);
        let str_lit: Lit = parse_quote!("str");
        let float_lit: Lit = parse_quote!(12.1);
        let bool_lit: Lit = parse_quote!(false);

        assert_eq!(Ok(12), u64::from_lit(int_lit));
        assert_eq!(Err(CastError), u64::from_lit(str_lit));
        assert_eq!(Err(CastError), u64::from_lit(float_lit));
        assert_eq!(Err(CastError), u64::from_lit(bool_lit));
    }

    #[test]
    fn test_str() {
        let int_lit: Lit = parse_quote!(12);
        let str_lit: Lit = parse_quote!("str");
        let float_lit: Lit = parse_quote!(12.1);
        let bool_lit: Lit = parse_quote!(false);

        assert_eq!(Err(CastError), String::from_lit(int_lit));
        assert_eq!(Ok("str".to_string()), String::from_lit(str_lit));
        assert_eq!(Err(CastError), String::from_lit(float_lit));
        assert_eq!(Err(CastError), String::from_lit(bool_lit));
    }

    #[test]
    fn test_float() {
        let int_lit: Lit = parse_quote!(12);
        let str_lit: Lit = parse_quote!("str");
        let float_lit: Lit = parse_quote!(12.1);
        let bool_lit: Lit = parse_quote!(false);

        assert_eq!(Err(CastError), f64::from_lit(int_lit));
        assert_eq!(Err(CastError), f64::from_lit(str_lit));
        assert_eq!(Ok(12.1), f64::from_lit(float_lit));
        assert_eq!(Err(CastError), f64::from_lit(bool_lit));
    }

    #[test]
    fn test_bool() {
        let int_lit: Lit = parse_quote!(12);
        let str_lit: Lit = parse_quote!("str");
        let float_lit: Lit = parse_quote!(12.1);
        let bool_lit: Lit = parse_quote!(false);

        assert_eq!(Err(CastError), bool::from_lit(int_lit));
        assert_eq!(Err(CastError), bool::from_lit(str_lit));
        assert_eq!(Err(CastError), bool::from_lit(float_lit));
        assert_eq!(Ok(false), bool::from_lit(bool_lit));
    }
}
