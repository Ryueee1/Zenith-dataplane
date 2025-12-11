use anyhow::Result;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct SchemaField {
    name: String,
    #[serde(rename = "type")]
    field_type: String,
    nullable: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SchemaDefinition {
    name: String,
    fields: Vec<SchemaField>,
}

pub fn generate(input: &Path, lang: &str, output: &Path) -> Result<()> {
    let content = fs::read_to_string(input)?;
    let schema: SchemaDefinition = serde_json::from_str(&content)?;
    
    let generated_code = match lang {
        "rust" => generate_rust_schema(&schema),
        "python" => generate_python_schema(&schema),
        _ => return Err(anyhow::anyhow!("Unsupported language: {}", lang)),
    };
    
    fs::write(output, generated_code)?;
    Ok(())
}

fn generate_rust_schema(schema: &SchemaDefinition) -> String {
    let mut code = format!("// Auto-generated schema for {}\n", schema.name);
    code.push_str("use arrow::datatypes::{{DataType, Field, Schema}};\n\n");
    code.push_str(&format!("pub fn {}_schema() -> Schema {{\n", schema.name.to_lowercase()));
    code.push_str("    Schema::new(vec![\n");
    
    for field in &schema.fields {
        let arrow_type = map_type_to_arrow(&field.field_type);
        let nullable = field.nullable.unwrap_or(false);
        code.push_str(&format!(
            "        Field::new(\"{}\", {}, {}),\n",
            field.name, arrow_type, nullable
        ));
    }
    
    code.push_str("    ])\n");
    code.push_str("}\n");
    code
}

fn generate_python_schema(schema: &SchemaDefinition) -> String {
    let mut code = format!("# Auto-generated schema for {}\n", schema.name);
    code.push_str("import pyarrow as pa\n\n");
    code.push_str(&format!("def {}_schema():\n", schema.name.to_lowercase()));
    code.push_str("    return pa.schema([\n");
    
    for field in &schema.fields {
        let py_type = map_type_to_pyarrow(&field.field_type);
        code.push_str(&format!(
            "        pa.field('{}', {}),\n",
            field.name, py_type
        ));
    }
    
    code.push_str("    ])\n");
    code
}

fn map_type_to_arrow(type_str: &str) -> String {
    match type_str {
        "string" => "DataType::Utf8".to_string(),
        "int32" => "DataType::Int32".to_string(),
        "int64" => "DataType::Int64".to_string(),
        "uint32" => "DataType::UInt32".to_string(),
        "uint64" => "DataType::UInt64".to_string(),
        "float32" => "DataType::Float32".to_string(),
        "float64" => "DataType::Float64".to_string(),
        "bool" => "DataType::Boolean".to_string(),
        _ => format!("DataType::Utf8 // Unknown: {}", type_str),
    }
}

fn map_type_to_pyarrow(type_str: &str) -> String {
    match type_str {
        "string" => "pa.string()".to_string(),
        "int32" => "pa.int32()".to_string(),
        "int64" => "pa.int64()".to_string(),
        "uint32" => "pa.uint32()".to_string(),
        "uint64" => "pa.uint64()".to_string(),
        "float32" => "pa.float32()".to_string(),
        "float64" => "pa.float64()".to_string(),
        "bool" => "pa.bool_()".to_string(),
        _ => format!("pa.string()  # Unknown: {}", type_str),
    }
}
