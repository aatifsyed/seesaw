#[test]
fn test() {
    let bindings = bindgen::builder()
        .header_contents(
            "yakshaver.h",
            include_str!("../examples/yakshaver/yakshaver.h"),
        )
        .generate()
        .unwrap();

    let mut seesaw = String::new();
    seesaw::seesaw("Yakshaver", &bindings, &mut seesaw).unwrap();

    let mut bindgen = Vec::new();
    bindings.write(Box::new(&mut bindgen)).unwrap();

    expect_test::expect_file!["../examples/yakshaver/generated/bindgen.rs"]
        .assert_eq(&String::from_utf8(bindgen).unwrap());
    expect_test::expect_file!["../examples/yakshaver/generated/seesaw.rs"].assert_eq(&seesaw);
}
