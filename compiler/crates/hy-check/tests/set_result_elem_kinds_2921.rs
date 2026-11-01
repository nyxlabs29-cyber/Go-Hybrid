//! Regression tests for #2921: `set[T]` method results, `for v in s`
//! element kinds, set arguments reached through an index, and set method
//! calls on unaddressable receivers. Each rejected program passed
//! `hyb check` and failed `go build` (go1.26.9) before the fix.

use hy_check::check_file;
use hy_syntax::parse_file;

fn codes(src: &str) -> Vec<String> {
    let f = parse_file("test.hy", src).expect("parse");
    check_file(&f).diagnostics.into_iter().map(|d| d.code).collect()
}

fn rejects(body: &str) {
    let src = format!("fn main() {{\n{}\n}}\n", body);
    let c = codes(&src);
    assert!(
        c.iter().any(|c| c == "HYB-E1002"),
        "expected HYB-E1002 for:\n{}\ngot {:?}",
        src,
        c
    );
}

fn accepts(src: &str) {
    let c = codes(src);
    assert!(c.is_empty(), "expected 0 diagnostics for:\n{}\ngot {:?}", src, c);
}

#[test]
fn insert_result_is_bool() {
    rejects("s := set[int]{}\nvar b string = s.insert(1)\nfmt.println(b)");
}

#[test]
fn contains_result_is_bool() {
    rejects("s := set[int]{}\nvar n int = s.contains(1)\nfmt.println(n)");
}

#[test]
fn len_method_result_is_int() {
    rejects("s := set[int]{}\nvar n string = s.len()\nfmt.println(n)");
}

#[test]
fn for_in_set_binds_element_kind() {
    rejects("s := set[int]{}\ns.insert(1)\nfor v in s {\nvar t string = v\nfmt.println(t)\n}");
}

#[test]
fn indexed_slice_of_sets_checks_argument() {
    rejects("ss := []set[int]{set[int]{}}\nss[0].insert(\"x\")\nfmt.println(len(ss))");
}

#[test]
fn typed_param_slice_of_sets_checks_argument() {
    let src = "fn f(ss []set[string]) {\n    ss[0].insert(1)\n}\nfn main() {}\n";
    assert!(codes(src).iter().any(|c| c == "HYB-E1002"), "{:?}", codes(src));
}

#[test]
fn map_element_set_method_is_rejected() {
    rejects("m := map[string]set[int]{}\nfmt.println(m[\"a\"].contains(1))");
}

#[test]
fn call_result_set_method_is_rejected() {
    let src = "fn mk() set[int] {\n    return set[int]{}\n}\nfn main() {\n    fmt.println(mk().contains(1))\n}\n";
    assert!(codes(src).iter().any(|c| c == "HYB-E1002"), "{:?}", codes(src));
}

#[test]
fn valid_set_uses_still_pass() {
    accepts(
        "fn main() {\n    s := set[int]{}\n    ok := s.insert(1)\n    var b bool = s.contains(1)\n    ss := []set[int]{set[int]{}}\n    ss[0].insert(2)\n    m := map[string]set[int]{}\n    t := m[\"a\"]\n    t.insert(3)\n    m[\"a\"] = t\n    for v in s {\n        var x int = v\n        fmt.println(x)\n    }\n    fmt.println(ok, b, ss[0].contains(2), m[\"a\"].len(), s.len()+1)\n}\n",
    );
}
