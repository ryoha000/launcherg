use super::path::resolve;

#[test]
fn path_入力がそのまま返る() {
    let p = resolve("C:/images/a.png");
    assert_eq!(p, "C:/images/a.png");
}
