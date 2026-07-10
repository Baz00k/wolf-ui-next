use std::env;
use std::fs;
use std::path::PathBuf;

use openapi_to_rust::{CodeGenerator, GeneratorConfig, SchemaAnalyzer};

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set"));
    let schema_path = manifest_dir.join("openapi/wolf.openapi.json");

    println!("cargo:rerun-if-changed={}", schema_path.display());

    let schema = fs::read_to_string(&schema_path).expect("read Wolf OpenAPI schema");
    let mut spec: serde_json::Value =
        serde_json::from_str(&schema).expect("parse Wolf OpenAPI schema JSON");
    normalize_numeric_constraints(&mut spec);

    let mut analyzer = SchemaAnalyzer::new(spec).expect("create Wolf OpenAPI analyzer");
    let mut analysis = analyzer.analyze().expect("analyze Wolf OpenAPI schema");

    let generator = CodeGenerator::new(GeneratorConfig {
        spec_path: schema_path,
        module_name: "generated".to_owned(),
        enable_async_client: false,
        enable_sse_client: false,
        tracing_enabled: false,
        ..GeneratorConfig::default()
    });

    let generated = generator
        .generate(&mut analysis)
        .expect("generate Wolf API types");
    let generated = format_generated_types(&generated);
    let generated = generated.replace(
        "#[derive(Debug, Clone, Deserialize, Serialize)]",
        "#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]",
    );

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR set"));
    fs::write(out_dir.join("types.rs"), generated).expect("write generated Wolf API types");
}

fn normalize_numeric_constraints(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                normalize_numeric_constraints(value);
            }
        }
        serde_json::Value::Object(values) => {
            for key in ["minItems", "maxItems"] {
                if let Some(serde_json::Value::String(value)) = values.get(key)
                    && let Ok(value) = value.parse::<u64>()
                {
                    values.insert(key.to_owned(), serde_json::Value::from(value));
                }
            }

            for value in values.values_mut() {
                normalize_numeric_constraints(value);
            }
        }
        _ => {}
    }
}

fn format_generated_types(generated: &str) -> String {
    let generated = generated
        .replace("//!", "//")
        .replace("#![allow(", "#[allow(");

    match syn::parse_file(&generated) {
        Ok(file) => prettyplease::unparse(&file),
        Err(_) => generated,
    }
}
