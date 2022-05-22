fn main() {
    // re build by changes [ build.rs, src/wire/wire.proto, Cargo.toml ]
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/wire/wire.proto");
    println!("cargo:rerun-if-changed=Cargo.toml");

    tonic_build::configure()
        // add serde
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"snake_case\")]")
        // key is proto field
        .field_attribute("send_message", "#[serde(rename = \"send_message\")]")
        .out_dir("src/wire")
        .compile(&["src/wire/wire.proto"], &["src/wire"])
        .unwrap();
}
