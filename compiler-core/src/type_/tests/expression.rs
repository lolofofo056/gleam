use ecow::EcoString;
use itertools::Itertools;

use crate::{analyse::TargetSupport, build::Target, type_::expression::Implementations};

use super::compile_module_with_target;

macro_rules! assert_targets {
    ($src:expr, $implementations:expr $(,)?) => {
        let result = $crate::type_::tests::expression::implementations($src);
        let expected = $implementations
            .iter()
            .map(|(name, expected_impl)| ((*name).into(), *expected_impl))
            .collect_vec();
        assert_eq!(expected, result);
    };
}

pub fn implementations(src: &str) -> Vec<(EcoString, Implementations)> {
    compile_module_with_target(
        src,
        None,
        vec![],
        Target::Erlang,
        TargetSupport::NotEnforced,
    )
    .expect("compile src")
    .type_info
    .values
    .into_iter()
    .map(|(name, value)| (name, value.variant.implementations()))
    .sorted()
    .collect_vec()
}

#[test]
pub fn pure_gleam_function() {
    assert_targets!(
        r#"
pub fn pure_gleam_1() { 1 + 1 }
pub fn pure_gleam_2() { pure_gleam_1() * 2 }
"#,
        vec![
            (
                "pure_gleam_1",
                Implementations {
                    gleam: true,
                    erlang: false,
                    javascript: false,
                }
            ),
            (
                "pure_gleam_2",
                Implementations {
                    gleam: true,
                    erlang: false,
                    javascript: false,
                }
            )
        ],
    );
}

#[test]
pub fn erlang_only_function() {
    assert_targets!(
        r#"
@external(erlang, "foo", "bar")
pub fn erlang_only_1() -> Int

pub fn erlang_only_2() { erlang_only_1() * 2 }
"#,
        vec![
            (
                "erlang_only_1",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: false,
                }
            ),
            (
                "erlang_only_2",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: false,
                }
            ),
        ],
    );
}

#[test]
pub fn externals_only_function() {
    assert_targets!(
        r#"
@external(erlang, "foo", "bar")
@external(javascript, "foo", "bar")
pub fn all_externals_1() -> Int

pub fn all_externals_2() { all_externals_1() * 2 }
"#,
        vec![
            (
                "all_externals_1",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: true,
                }
            ),
            (
                "all_externals_2",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: true,
                }
            )
        ],
    );
}

#[test]
pub fn externals_with_pure_gleam_body() {
    assert_targets!(
        r#"
@external(javascript, "foo", "bar")
pub fn javascript_external_and_pure_body() -> Int { 1 + 1 }

@external(erlang, "foo", "bar")
pub fn erlang_external_and_pure_body() -> Int { 1 + 1 }

pub fn pure_gleam() {
  javascript_external_and_pure_body() + erlang_external_and_pure_body()
}
"#,
        vec![
            (
                "erlang_external_and_pure_body",
                Implementations {
                    gleam: true,
                    erlang: true,
                    javascript: false,
                }
            ),
            (
                "javascript_external_and_pure_body",
                Implementations {
                    gleam: true,
                    erlang: false,
                    javascript: true,
                }
            ),
            (
                "pure_gleam",
                Implementations {
                    gleam: true,
                    erlang: true,
                    javascript: true,
                }
            )
        ],
    );
}

#[test]
pub fn erlang_external_with_javascript_body() {
    assert_targets!(
        r#"
@external(javascript, "foo", "bar")
pub fn javascript_only() -> Int

@external(erlang, "foo", "bar")
pub fn erlang_external_and_javascript_body() -> Int { javascript_only() }

pub fn all_externals() -> Int { erlang_external_and_javascript_body() }
"#,
        vec![
            (
                "all_externals",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: true,
                }
            ),
            (
                "erlang_external_and_javascript_body",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: true,
                }
            ),
            (
                "javascript_only",
                Implementations {
                    gleam: false,
                    erlang: false,
                    javascript: true,
                }
            ),
        ],
    );
}

#[test]
pub fn javascript_external_with_erlang_body() {
    assert_targets!(
        r#"
@external(erlang, "foo", "bar")
pub fn erlang_only() -> Int

@external(javascript, "foo", "bar")
pub fn javascript_external_and_erlang_body() -> Int { erlang_only() }

pub fn all_externals() -> Int { javascript_external_and_erlang_body() }
"#,
        vec![
            (
                "all_externals",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: true,
                }
            ),
            (
                "erlang_only",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: false,
                }
            ),
            (
                "javascript_external_and_erlang_body",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: true,
                }
            ),
        ],
    );
}

#[test]
pub fn function_with_no_valid_implementations() {
    assert_targets!(
        r#"
@external(javascript, "foo", "bar")
pub fn javascript_only() -> Int
        
@external(erlang, "foo", "bar")
pub fn erlang_only() -> Int

pub fn main() {
    javascript_only()
    erlang_only()
}
"#,
        vec![
            (
                "erlang_only",
                Implementations {
                    gleam: false,
                    erlang: true,
                    javascript: false,
                }
            ),
            (
                "javascript_only",
                Implementations {
                    gleam: false,
                    erlang: false,
                    javascript: true,
                }
            ),
            (
                "no_implementations",
                Implementations {
                    gleam: false,
                    erlang: false,
                    javascript: false,
                }
            )
        ],
    );
}