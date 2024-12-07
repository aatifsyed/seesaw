// this is all duplicated because I can't get `cargo release` to include the examples when packaging
// and I don't want users to have to follow symlinks.
#[test]
fn test() {
    let bindings = bindgen::builder()
        .header_contents("yakshaver.h", include_str!("yakshaver.h"))
        .generate()
        .unwrap();

    let mut seesaw = String::new();
    seesaw::seesaw("Yakshaver", &bindings, &mut seesaw).unwrap();

    let mut bindgen = Vec::new();
    bindings.write(Box::new(&mut bindgen)).unwrap();

    expect_test::expect_file!["generated/bindgen.rs"]
        .assert_eq(&String::from_utf8(bindgen).unwrap());
    expect_test::expect_file!["generated/seesaw.rs"].assert_eq(&seesaw);
}
