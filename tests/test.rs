use seesaw::Trait;

#[test]
fn test() {
    let bindings = bindgen::builder()
        .header_contents("yakshaver.h", include_str!("doc/yakshaver.h"))
        .generate()
        .unwrap();
    let mut dest = String::new();
    seesaw::seesaw(Trait::new("YakShaver"), bindings, &mut dest).unwrap();
    expect_test::expect_file!["doc/yakshaver.rs"].assert_eq(&dest);
}
