[![Build Status](https://www.travis-ci.org/boncheolgu/syn_utils.svg?branch=master)](https://www.travis-ci.org/boncheolgu/syn_util)

# Helper APIs for Rust procedural macros

This crates provides helper APIs for procedural macros.
Please read the [`API documentation on docs.rs`](https://docs.rs/syn_util)

## APIs to manipulate with attributes

```rust
use proc_macro2::Span;
use syn::LitStr;

fn lit_str(s: &str) -> Lit {
    Lit::Str(LitStr::new(s, Span::call_site()))
}

fn test_contains_attribute_impl() {
    let attr: Attribute = parse_quote!(#[level0(level1, level1_1(level2, level2_1 = "hello"))]);
    let attr = [attr];

    assert!(!contains_attribute(&attr, &[]));

    assert!(!contains_attribute(&attr, &["not"]));

    assert!(!contains_attribute(&attr, &["level0"]));

    assert!(contains_attribute(&attr, &["level0", "level1"]));

    assert!(!contains_attribute(&attr, &["level0", "level1_1"]),);

    assert!(contains_attribute(&attr, &["level0", "level1_1", "level2"]),);

    assert!(!contains_attribute(
        &attr,
        &["level0", "level1_1", "level2_1"]
    ),);
}

#[test]
fn test_get_attribute_value() {
    let attr: Attribute = parse_quote!(#[level0(level1 = "hi", level1_1(level2 = "bye"))]);
    let attr = [attr];

    assert_eq!(get_attribute_value(&attr, &[""]), None);

    assert_eq!(get_attribute_value(&attr, &["not"]), None);

    assert_eq!(get_attribute_value(&attr, &["level0"]), None);

    assert_eq!(
        get_attribute_value(&attr, &["level0", "level1"]),
        Some(lit_str("hi"))
    );

    assert_eq!(get_attribute_value(&attr, &["level0", "level1_1"]), None);

    assert_eq!(
        get_attribute_value(&attr, &["level0", "level1_1", "level2"]),
        Some(lit_str("bye"))
    );
}

#[test]
fn test_get_attribute_map() {
    assert_eq!(
        get_attribute_map(
            &[
                parse_quote!(#[level9]),
                parse_quote!(#[level0(level8)]),
                parse_quote!(#[level0(level1 = "hi", level1_1(level2 = "bye"))]),
                parse_quote!(#[level0(level1 = "hi", level1_1(level2 = "bye"))]),
                parse_quote!(#[gen0(gen1 = "amoeba", gen1_1 = "monad", gen1_2(gen2 = "monoid"))])
            ],
            "."
        ),
        vec![
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
```
