mod lit_cast;

use std::collections::HashMap;

use pmutil::ToTokensExt;
use syn::{AttrStyle, Attribute, Lit, Meta, MetaNameValue, NestedMeta};

use crate::lit_cast::FromLit;

pub fn contains_attribute(attrs: &[Attribute], id: &[&str]) -> bool {
    for attr in attrs {
        if attr.style != AttrStyle::Outer {
            continue;
        }

        if let Ok(meta) = attr.parse_meta() {
            if contains_attribute_impl(&meta, &id) {
                return true;
            }
        }
    }

    false
}

fn contains_attribute_impl<'a>(meta: &'a Meta, id: &[&str]) -> bool {
    if id.is_empty() {
        return false;
    }

    match meta {
        Meta::Path(path) => id.len() == 1 && path.is_ident(id[0]),
        Meta::List(meta_list) if meta_list.path.is_ident(id[0]) => {
            for nested_meta in &meta_list.nested {
                if let NestedMeta::Meta(meta) = nested_meta {
                    if contains_attribute_impl(meta, &id[1..]) {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

pub fn get_attribute_value<T: FromLit>(attrs: &[Attribute], id: &[&str]) -> Option<T> {
    for attr in attrs {
        if attr.style != AttrStyle::Outer {
            continue;
        }

        if let Ok(meta) = attr.parse_meta() {
            if let Some(value) = get_attribute_value_impl(&meta, &id) {
                if let Ok(parsed) = T::from_lit(value.clone()) {
                    return Some(parsed);
                }
            }
        }
    }

    None
}

fn get_attribute_value_impl<'a>(meta: &'a Meta, id: &[&str]) -> Option<&'a Lit> {
    if id.is_empty() {
        return None;
    }

    match meta {
        Meta::NameValue(MetaNameValue { path, lit, .. }) if path.is_ident(id[0]) => Some(lit),
        Meta::List(meta_list) if meta_list.path.is_ident(id[0]) => {
            for nested_meta in &meta_list.nested {
                if let NestedMeta::Meta(meta) = nested_meta {
                    let r = get_attribute_value_impl(meta, &id[1..]);
                    if r.is_some() {
                        return r;
                    }
                }
            }
            None
        }
        _ => None,
    }
}

pub fn get_attribute_map(attrs: &[Attribute], separator: &str) -> HashMap<String, Vec<Lit>> {
    let mut result = HashMap::new();

    for attr in attrs {
        if attr.style != AttrStyle::Outer {
            continue;
        }

        if let Ok(meta) = attr.parse_meta() {
            get_attribute_map_impl(&mut result, &meta, "", separator);
        }
    }

    result
}

fn get_attribute_map_impl<'a>(
    map: &mut HashMap<String, Vec<Lit>>,
    meta: &Meta,
    prefix: &str,
    separator: &str,
) {
    match meta {
        Meta::Path(path) => {
            let key = if prefix.is_empty() {
                path.dump().to_string()
            } else {
                format!("{}{}{}", prefix, separator, path.dump())
            };
            assert!(!map.contains_key(&key), "{} already exists.", key);
            map.insert(key, vec![]);
        }
        Meta::NameValue(MetaNameValue { path, lit, .. }) => {
            let key = if prefix.is_empty() {
                path.dump().to_string()
            } else {
                format!("{}{}{}", prefix, separator, path.dump())
            };
            map.get_mut(&key)
                .map(|value| {
                    assert!(!value.is_empty(), "conflicts among `{}` attributes.", key);
                    value.push(lit.clone());
                })
                .unwrap_or_else(|| {
                    map.insert(key, vec![lit.clone()]);
                })
        }
        Meta::List(meta_list) => {
            let key = if prefix.is_empty() {
                meta_list.path.dump().to_string()
            } else {
                format!("{}{}{}", prefix, separator, meta_list.path.dump())
            };
            for nested_meta in &meta_list.nested {
                if let NestedMeta::Meta(meta) = nested_meta {
                    get_attribute_map_impl(map, &meta, &key, separator);
                }
            }
        }
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

        assert!(contains_attribute(&attr, &["level0", "level1_1", "level2_2"],),);

        assert!(!contains_attribute(&attr, &["level0", "level1_1", "level2_1"],),);
    }

    #[test]
    fn test_get_attribute_value_impl() {
        let attr: Attribute = parse_quote!(#[level0(level1 = "hi", level1_1(level2 = "bye"))]);

        let meta = attr.parse_meta().unwrap();

        assert_eq!(get_attribute_value_impl(&meta, &[]), None);

        assert_eq!(get_attribute_value_impl(&meta, &["not"]), None);

        assert_eq!(get_attribute_value_impl(&meta, &["level0"]), None);

        assert_eq!(get_attribute_value_impl(&meta, &["level0", "level1"]), Some(&lit_str("hi")));

        assert_eq!(get_attribute_value_impl(&meta, &["level0", "level1_1"]), None);

        assert_eq!(
            get_attribute_value_impl(&meta, &["level0", "level1_1", "level2"]),
            Some(&lit_str("bye"))
        );

        let attr: Attribute = parse_quote!(#[doc = "hi"]);

        let meta = attr.parse_meta().unwrap();

        assert_eq!(get_attribute_value_impl(&meta, &["doc"]), Some(&lit_str("hi")));
    }

    #[test]
    fn test_get_attribute_value() {
        let attr: Attribute = parse_quote!(#[level0 = "hi"]);
        assert_eq!(get_attribute_value(&[attr], &["level0"]), Some(lit_str("hi")));

        let attr: Attribute = parse_quote!(#[level0(level1 = "hi", level1_1(level2 = false))]);
        let attr = [attr];

        assert_eq!(get_attribute_value::<String>(&attr, &[""]), None);

        assert_eq!(get_attribute_value::<String>(&attr, &["not"]), None);

        assert_eq!(get_attribute_value::<String>(&attr, &["level0"]), None);

        assert_eq!(get_attribute_value(&attr, &["level0", "level1"]), Some("hi".to_string()));

        assert_eq!(get_attribute_value::<Lit>(&attr, &["level0", "level1_1"]), None);

        assert_eq!(get_attribute_value(&attr, &["level0", "level1_1", "level2"]), Some(false));
    }

    #[test]
    fn test_get_attribute_map_impl() {
        let attr: Attribute =
            parse_quote!(#[level0(level1 = "hi", level1 = "hi", level1_1(level2 = "bye"))]);

        let meta = attr.parse_meta().unwrap();

        let mut result = HashMap::new();
        get_attribute_map_impl(&mut result, &meta, "", ".");
        assert_eq!(
            result,
            vec![
                ("level0.level1".to_string(), vec![lit_str("hi"), lit_str("hi")],),
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
                ("level0.level1".to_string(), vec![lit_str("hi"), lit_str("hi")],),
                ("level0.level1_1.level2".to_string(), vec![lit_str("bye"), lit_str("bye")],),
                ("gen0.gen1".to_string(), vec![lit_str("amoeba")]),
                ("gen0.gen1_1".to_string(), vec![lit_str("monad")]),
                ("gen0.gen1_2.gen2".to_string(), vec![lit_str("monoid")]),
            ]
            .into_iter()
            .collect()
        );
    }
}
