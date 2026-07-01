use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use syn::{File as SynFile, Item};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_slint_types.slint");
    let mut slint_file = File::create(&dest_path).unwrap();

    let mut file_content = String::new();
    let mut f = File::open("src/lib.rs").unwrap();
    f.read_to_string(&mut file_content).unwrap();

    let ast: SynFile = syn::parse_file(&file_content).unwrap();

    for item in ast.items {
        if let Item::Struct(item_struct) = item {
            let has_oxide_slint_derive = item_struct.attrs.iter().any(|attr| {
                if let Ok(meta) = attr.meta.require_list() {
                    meta.path.is_ident("derive") && meta.tokens.to_string().contains("OxideSlint")
                } else {
                    false
                }
            });

            if has_oxide_slint_derive {
                writeln!(slint_file, "export struct {} {{", item_struct.ident).unwrap();
                for field in item_struct.fields.iter() {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = &field.ty;
                    let type_str = quote::quote!(#field_type).to_string();
                    let slint_type = match type_str.as_str() {
                        "u32" | "i32" => "int",
                        "f32" => "float",
                        "bool" => "bool",
                        "String" | "& str" => "string",
                        _ => "string", // Default to string for complex types
                    };
                    writeln!(slint_file, "    {}: {},", field_name, slint_type).unwrap();
                }
                writeln!(slint_file, "}}").unwrap();
            }
        }
    }
}
