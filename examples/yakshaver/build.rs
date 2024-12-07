fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bindings = bindgen::builder().header("yakshaver.h").generate()?;
    seesaw::seesaw("Yakshaver", &bindings, "generated/seesaw.rs")?;
    bindings.write_to_file("generated/bindgen.rs")?;
    Ok(())
}
