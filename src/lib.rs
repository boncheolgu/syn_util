mod lit_cast;

use pmutil::ToTokensExt;
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::{AttrStyle, Attribute, Expr, ExprLit, Lit, Meta, MetaList, MetaNameValue, Result, Token};

use crate::lit_cast::FromLit;

fn check_and_pop_hd<'a>(meta: &Meta, id: &'a [&'a str]) -> Option<&'a [&'a str]> {
    id.split_first().and_then(|(hd, tl)| {
        if meta.path().is_ident(hd) {
            Some(tl)
        } else {
            None
        }
    })
}

fn iter_meta_list<T, F>(meta_list: &MetaList, mut f: F) -> Result<T>
where
    F: FnMut(&mut syn::punctuated::Iter<Meta>) -> T,
{
    meta_list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .map(|nested_metas| f(&mut nested_metas.iter()))
}

pub fn contains_attribute(attrs: &[Attribute], id: &[&str]) -> bool {
    attrs.iter().any(|Attribute { style, meta, .. }| {
        *style == AttrStyle::Outer && contains_attribute_impl(meta, id)
    })
}

fn contains_attribute_impl(meta: &Meta, id: &[&str]) -> bool {
    let id = match check_and_pop_hd(meta, id) {
        Some(id) => id,
        None => {
            return false;
        }
    };

    match meta {
        Meta::Path(..) => id.is_empty(),
        Meta::List(meta_list) => iter_meta_list(meta_list, |iter| {
            iter.any(|meta| contains_attribute_impl(meta, id))
        })
        .unwrap_or(false),
        Meta::NameValue(..) => false,
    }
}

pub fn get_attribute_value<T: FromLit>(attrs: &[Attribute], id: &[&str]) -> Option<T> {
    attrs.iter().find_map(|Attribute { style, meta, .. }| {
        if *style != AttrStyle::Outer {
            return None;
        }
        get_attribute_value_impl(meta, id).and_then(|value| T::from_lit(value).ok())
    })
}

fn get_attribute_value_impl(meta: &Meta, id: &[&str]) -> Option<Lit> {
    let id = match check_and_pop_hd(meta, id) {
        Some(id) => id,
        None => {
            return None;
        }
    };

    match meta {
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit { lit, .. }),
            ..
        }) if id.is_empty() => Some(lit.clone()),
        Meta::List(meta_list) => iter_meta_list(meta_list, |iter| {
            iter.find_map(|meta| get_attribute_value_impl(meta, id))
        })
        .unwrap_or(None),
        Meta::Path(..) | Meta::NameValue(..) => None,
    }
}

pub fn get_attribute_map(attrs: &[Attribute], separator: &str) -> HashMap<String, Vec<Lit>> {
    let mut result = HashMap::new();
    attrs.iter().for_each(|Attribute { style, meta, .. }| {
        if *style == AttrStyle::Outer {
            get_attribute_map_impl(&mut result, meta, "", separator);
        }
    });
    result
}

fn get_attribute_map_impl(
    map: &mut HashMap<String, Vec<Lit>>,
    meta: &Meta,
    prefix: &str,
    separator: &str,
) -> () {
    let key = {
        let path = meta.path().dump();
        if prefix.is_empty() {
            path.to_string()
        } else {
            format!("{}{}{}", prefix, separator, path)
        }
    };

    match meta {
        Meta::Path(..) => {
            assert!(!map.contains_key(&key), "{} already exists.", key);
            map.insert(key, vec![]);
        }
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit { lit, .. }),
            ..
        }) => map
            .get_mut(&key)
            .map(|values| {
                assert!(!values.is_empty(), "conflicts among `{}` attributes.", key);
                values.push(lit.clone());
            })
            .unwrap_or_else(|| {
                map.insert(key, vec![lit.clone()]);
            }),
        Meta::NameValue(..) => (),
        Meta::List(meta_list) => iter_meta_list(meta_list, |iter| {
            iter.for_each(|meta| get_attribute_map_impl(map, meta, &key, separator))
        })
        .unwrap_or(()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proc_macro2::Span;
    use syn::{parse_quote, LitStr};

    fn lit_str(s: &str) -> Lit {
        Lit::Str(LitStr::new(s, Span::call_site()))
    }

    #[test]
    fn test_contains_attribute_impl() {
        let attr: Attribute = parse_quote!(#[level0]);
        assert!(contains_attribute(&[attr], &["level0"]));

        let attr: Attribute = parse_quote!(#[level0(level1, level1_1(level2, level2_1 = "hello", level2_2), level1_2)]);
        let attr = [attr];

        assert!(!contains_attribute(&attr, &[]));

        assert!(!contains_attribute(&attr, &["not"]));

        assert!(!contains_attribute(&attr, &["level0"]));

        assert!(contains_attribute(&attr, &["level0", "level1"]));

        assert!(!contains_attribute(&attr, &["level0", "level1_1"]));

        assert!(contains_attribute(&attr, &["level0", "level1_2"]));

        assert!(contains_attribute(&attr, &["level0", "level1_1", "level2"]));

        assert!(contains_attribute(
            &attr,
            &["level0", "level1_1", "level2_2"],
        ),);

        assert!(!contains_attribute(
            &attr,
            &["level0", "level1_1", "level2_1"],
        ),);
    }

    #[test]
    fn test_get_attribute_value_impl() {
        let attr: Attribute = parse_quote!(#[level0(level1 = "hi", level1_1(level2 = "bye"))]);

        let meta = attr.meta;

        assert_eq!(get_attribute_value_impl(&meta, &[]), None);

        assert_eq!(get_attribute_value_impl(&meta, &["not"]), None);

        assert_eq!(get_attribute_value_impl(&meta, &["level0"]), None);

        assert_eq!(
            get_attribute_value_impl(&meta, &["level0", "level1"]),
            Some(lit_str("hi"))
        );

        assert_eq!(
            get_attribute_value_impl(&meta, &["level0", "level1_1"]),
            None
        );

        assert_eq!(
            get_attribute_value_impl(&meta, &["level0", "level1_1", "level2"]),
            Some(lit_str("bye"))
        );

        let attr: Attribute = parse_quote!(#[doc = "hi"]);

        let meta = attr.meta;

        assert_eq!(
            get_attribute_value_impl(&meta, &["doc"]),
            Some(lit_str("hi"))
        );
    }

    #[test]
    fn test_get_attribute_value() {
        let attr: Attribute = parse_quote!(#[level0 = "hi"]);
        assert_eq!(
            get_attribute_value(&[attr], &["level0"]),
            Some(lit_str("hi"))
        );

        let attr: Attribute = parse_quote!(#[level0(level1 = "hi", level1_1(level2 = false))]);
        let attr = [attr];

        assert_eq!(get_attribute_value::<String>(&attr, &[""]), None);

        assert_eq!(get_attribute_value::<String>(&attr, &["not"]), None);

        assert_eq!(get_attribute_value::<String>(&attr, &["level0"]), None);

        assert_eq!(
            get_attribute_value(&attr, &["level0", "level1"]),
            Some("hi".to_string())
        );

        assert_eq!(
            get_attribute_value::<Lit>(&attr, &["level0", "level1_1"]),
            None
        );

        assert_eq!(
            get_attribute_value(&attr, &["level0", "level1_1", "level2"]),
            Some(false)
        );
    }

    #[test]
    fn test_get_attribute_map_impl() {
        let attr: Attribute =
            parse_quote!(#[level0(level1 = "hi", level1 = "hi", level1_1(level2 = "bye"))]);

        let meta = attr.meta;

        let mut result = HashMap::new();
        get_attribute_map_impl(&mut result, &meta, "", ".");
        assert_eq!(
            result,
            vec![
                (
                    "level0.level1".to_string(),
                    vec![lit_str("hi"), lit_str("hi")],
                ),
                ("level0.level1_1.level2".to_string(), vec![lit_str("bye")]),
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn test_get_attribute_map() {
        assert_eq!(
            get_attribute_map(
                &[
                    parse_quote!(#[level9]),
                    parse_quote!(#[level0_0 = "greeting"]),
                    parse_quote!(#[level0(level8)]),
                    parse_quote!(#[level0(level1 = "hi", level1_1(level2 = "bye"))]),
                    parse_quote!(#[level0(level1 = "hi", level1_1(level2 = "bye"))]),
                    parse_quote!(#[gen0(gen1 = "amoeba", gen1_1 = "monad", gen1_2(gen2 = "monoid"))])
                ],
                ".",
            ),
            vec![
                ("level0_0".to_string(), vec![lit_str("greeting")]),
                ("level9".to_string(), vec![]),
                ("level0.level8".to_string(), vec![]),
                (
                    "level0.level1".to_string(),
                    vec![lit_str("hi"), lit_str("hi")],
                ),
                (
                    "level0.level1_1.level2".to_string(),
                    vec![lit_str("bye"), lit_str("bye")],
                ),
                ("gen0.gen1".to_string(), vec![lit_str("amoeba")]),
                ("gen0.gen1_1".to_string(), vec![lit_str("monad")]),
                ("gen0.gen1_2.gen2".to_string(), vec![lit_str("monoid")]),
            ]
            .into_iter()
            .collect()
        );
    }
}
