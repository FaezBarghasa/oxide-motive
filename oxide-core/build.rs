use std::env;
use std::fs;
use std::path::Path;
use syn::{File, Item, Type, Ident};
use quote::quote;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = Path::new(&crate_dir).join("src").join("lib.rs");
    let content = fs::read_to_string(src_path).expect("Could not read src/lib.rs");

    let ast: File = syn::parse_file(&content).expect("Could not parse src/lib.rs");

    let mut slint_structs = String::new();

    for item in &ast.items {
        if let Item::Struct(item_struct) = item {
            if item_struct.attrs.iter().any(|attr| attr.path().is_ident("OxideSlint")) {
                let struct_name = &item_struct.ident;
                let mut slint_fields = String::new();

                for field in &item_struct.fields {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = &field.ty;
                    let slint_type = rust_to_slint_type(field_type);
                    slint_fields.push_str(&format!("    {}: {},\n", field_name, slint_type));
                }

                slint_structs.push_str(&format!(
                    "export struct {} {{\n{}}}\n\n",
                    struct_name, slint_fields
                ));
            }
        }
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.slint");
    fs::write(dest_path, slint_structs).unwrap();

    println!("cargo:rerun-if-changed=src/lib.rs");
}

fn rust_to_slint_type(ty: &Type) -> String {
    let type_str = quote!(#ty).to_string();
    match type_str.as_str() {
        "f32" | "f64" => "float".to_string(),
        "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => "int".to_string(),
        "bool" => "bool".to_string(),
        "String" | "& str" => "string".to_string(),
        _ => "string".to_string(), // Default to string for complex types
    }
}