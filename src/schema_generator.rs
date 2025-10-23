use crate::extractor::{Parameter, ParameterLocation, TypeInfo};
use crate::type_resolver::{PrimitiveType, TypeKind, TypeResolver};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema generator - converts Rust types to OpenAPI schemas
pub struct SchemaGenerator {
    /// Type resolver for looking up type definitions
    type_resolver: TypeResolver,
    /// Cache of generated schemas to avoid duplication
    schemas: HashMap<String, Schema>,
}

/// OpenAPI Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// The type of the schema (string, integer, object, array, etc.)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    /// Properties for object types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Property>>,
    /// Required field names for object types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Items schema for array types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    /// Enum values for enum types
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// Reference to another schema
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// Format for primitive types (e.g., "int32", "int64", "float", "double")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Property definition for object schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    /// The type of the property
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub property_type: Option<String>,
    /// Reference to another schema
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// Items schema for array properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    /// Format for primitive types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Parameter schema for OpenAPI parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Parameter name
    pub name: String,
    /// Parameter location (path, query, header)
    #[serde(rename = "in")]
    pub location: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Schema for the parameter
    pub schema: Schema,
}

impl SchemaGenerator {
    /// Create a new SchemaGenerator with a TypeResolver
    pub fn new(type_resolver: TypeResolver) -> Self {
        debug!("Initializing SchemaGenerator");
        Self {
            type_resolver,
            schemas: HashMap::new(),
        }
    }

    /// Generate a schema for a TypeInfo
    pub fn generate_schema(&mut self, type_info: &TypeInfo) -> Schema {
        debug!("Generating schema for type: {}", type_info.name);

        // Handle Option<T> - unwrap and generate schema for inner type
        if type_info.is_option {
            if let Some(inner) = type_info.generic_args.first() {
                return self.generate_schema(inner);
            }
        }

        // Handle Vec<T> - generate array schema
        if type_info.is_vec {
            if let Some(inner) = type_info.generic_args.first() {
                let items_schema = self.generate_schema(inner);
                return Schema {
                    schema_type: Some("array".to_string()),
                    properties: None,
                    required: None,
                    items: Some(Box::new(items_schema)),
                    enum_values: None,
                    reference: None,
                    format: None,
                };
            }
        }

        // Try to resolve as a primitive type first
        if let Some(resolved) = self.type_resolver.resolve_type(&type_info.name) {
            match resolved.kind {
                TypeKind::Primitive(prim) => {
                    return self.primitive_to_schema(&prim);
                }
                TypeKind::Struct(_) => {
                    // For structs, return a reference and ensure the schema is generated
                    self.generate_struct_schema(&type_info.name);
                    return Schema {
                        schema_type: None,
                        properties: None,
                        required: None,
                        items: None,
                        enum_values: None,
                        reference: Some(format!("#/components/schemas/{}", type_info.name)),
                        format: None,
                    };
                }
                TypeKind::Enum(_) => {
                    // For enums, return a reference and ensure the schema is generated
                    self.generate_enum_schema(&type_info.name);
                    return Schema {
                        schema_type: None,
                        properties: None,
                        required: None,
                        items: None,
                        enum_values: None,
                        reference: Some(format!("#/components/schemas/{}", type_info.name)),
                        format: None,
                    };
                }
                TypeKind::Generic(_) => {
                    // Generic types - use a placeholder
                    return Schema {
                        schema_type: Some("object".to_string()),
                        properties: None,
                        required: None,
                        items: None,
                        enum_values: None,
                        reference: None,
                        format: None,
                    };
                }
            }
        }

        // Fallback for unknown types
        debug!("Unknown type: {}, using object placeholder", type_info.name);
        Schema {
            schema_type: Some("object".to_string()),
            properties: None,
            required: None,
            items: None,
            enum_values: None,
            reference: None,
            format: None,
        }
    }

    /// Convert a primitive type to an OpenAPI schema
    fn primitive_to_schema(&self, primitive: &PrimitiveType) -> Schema {
        let (schema_type, format) = match primitive {
            PrimitiveType::String => ("string", None),
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 => {
                ("integer", Some("int32"))
            }
            PrimitiveType::I64 | PrimitiveType::I128 => ("integer", Some("int64")),
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 => {
                ("integer", Some("int32"))
            }
            PrimitiveType::U64 | PrimitiveType::U128 => ("integer", Some("int64")),
            PrimitiveType::F32 => ("number", Some("float")),
            PrimitiveType::F64 => ("number", Some("double")),
            PrimitiveType::Bool => ("boolean", None),
            PrimitiveType::Char => ("string", None),
        };

        Schema {
            schema_type: Some(schema_type.to_string()),
            properties: None,
            required: None,
            items: None,
            enum_values: None,
            reference: None,
            format: format.map(|s| s.to_string()),
        }
    }

    /// Generate a schema for a struct type and add it to the schemas collection
    fn generate_struct_schema(&mut self, type_name: &str) {
        // Check if already generated
        if self.schemas.contains_key(type_name) {
            debug!("Schema for {} already exists", type_name);
            return;
        }

        debug!("Generating struct schema for: {}", type_name);

        // Resolve the type
        let resolved = match self.type_resolver.resolve_type(type_name) {
            Some(r) => r,
            None => {
                debug!("Could not resolve type: {}", type_name);
                return;
            }
        };

        if let TypeKind::Struct(struct_def) = resolved.kind {
            let mut properties = HashMap::new();
            let mut required = Vec::new();

            for field in &struct_def.fields {
                // Skip fields marked with #[serde(skip)]
                if field.serde_attrs.skip {
                    continue;
                }

                // Use the renamed field name if specified
                let field_name = field
                    .serde_attrs
                    .rename
                    .as_ref()
                    .unwrap_or(&field.name)
                    .clone();

                // Generate property schema
                let property = self.type_info_to_property(&field.type_info);
                properties.insert(field_name.clone(), property);

                // Add to required list if not optional
                if !field.optional && !field.type_info.is_option {
                    required.push(field_name);
                }
            }

            let schema = Schema {
                schema_type: Some("object".to_string()),
                properties: Some(properties),
                required: if required.is_empty() {
                    None
                } else {
                    Some(required)
                },
                items: None,
                enum_values: None,
                reference: None,
                format: None,
            };

            self.schemas.insert(type_name.to_string(), schema);
        }
    }

    /// Generate a schema for an enum type and add it to the schemas collection
    fn generate_enum_schema(&mut self, type_name: &str) {
        // Check if already generated
        if self.schemas.contains_key(type_name) {
            debug!("Schema for {} already exists", type_name);
            return;
        }

        debug!("Generating enum schema for: {}", type_name);

        // Resolve the type
        let resolved = match self.type_resolver.resolve_type(type_name) {
            Some(r) => r,
            None => {
                debug!("Could not resolve type: {}", type_name);
                return;
            }
        };

        if let TypeKind::Enum(enum_def) = resolved.kind {
            let schema = Schema {
                schema_type: Some("string".to_string()),
                properties: None,
                required: None,
                items: None,
                enum_values: Some(enum_def.variants),
                reference: None,
                format: None,
            };

            self.schemas.insert(type_name.to_string(), schema);
        }
    }

    /// Convert a TypeInfo to a Property
    fn type_info_to_property(&mut self, type_info: &TypeInfo) -> Property {
        // Handle Option<T> - unwrap and generate property for inner type
        if type_info.is_option {
            if let Some(inner) = type_info.generic_args.first() {
                return self.type_info_to_property(inner);
            }
        }

        // Handle Vec<T> - generate array property
        if type_info.is_vec {
            if let Some(inner) = type_info.generic_args.first() {
                let items_schema = self.generate_schema(inner);
                return Property {
                    property_type: Some("array".to_string()),
                    reference: None,
                    items: Some(Box::new(items_schema)),
                    format: None,
                };
            }
        }

        // Try to resolve the type
        if let Some(resolved) = self.type_resolver.resolve_type(&type_info.name) {
            match resolved.kind {
                TypeKind::Primitive(prim) => {
                    let schema = self.primitive_to_schema(&prim);
                    return Property {
                        property_type: schema.schema_type,
                        reference: None,
                        items: None,
                        format: schema.format,
                    };
                }
                TypeKind::Struct(_) => {
                    // Generate the struct schema if not already done
                    self.generate_struct_schema(&type_info.name);
                    return Property {
                        property_type: None,
                        reference: Some(format!("#/components/schemas/{}", type_info.name)),
                        items: None,
                        format: None,
                    };
                }
                TypeKind::Enum(_) => {
                    // Generate the enum schema if not already done
                    self.generate_enum_schema(&type_info.name);
                    return Property {
                        property_type: None,
                        reference: Some(format!("#/components/schemas/{}", type_info.name)),
                        items: None,
                        format: None,
                    };
                }
                TypeKind::Generic(_) => {
                    return Property {
                        property_type: Some("object".to_string()),
                        reference: None,
                        items: None,
                        format: None,
                    };
                }
            }
        }

        // Fallback for unknown types
        Property {
            property_type: Some("object".to_string()),
            reference: None,
            items: None,
            format: None,
        }
    }

    /// Generate a parameter schema from a Parameter
    pub fn generate_parameter_schema(&mut self, param: &Parameter) -> ParameterSchema {
        debug!("Generating parameter schema for: {}", param.name);

        let location = match param.location {
            ParameterLocation::Path => "path",
            ParameterLocation::Query => "query",
            ParameterLocation::Header => "header",
        };

        let schema = self.generate_schema(&param.type_info);

        ParameterSchema {
            name: param.name.clone(),
            location: location.to_string(),
            required: param.required,
            schema,
        }
    }

    /// Get all generated schemas
    pub fn get_schemas(&self) -> &HashMap<String, Schema> {
        &self.schemas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::AstParser;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create a temporary file with content
    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    /// Helper function to create a SchemaGenerator from code
    fn create_generator_from_code(code: &str) -> SchemaGenerator {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "test.rs", code);
        let parsed = AstParser::parse_file(&file_path).unwrap();
        let type_resolver = TypeResolver::new(vec![parsed]);
        SchemaGenerator::new(type_resolver)
    }

    #[test]
    fn test_primitive_type_string() {
        let mut generator = create_generator_from_code("");
        let type_info = TypeInfo::new("String".to_string());
        let schema = generator.generate_schema(&type_info);

        assert_eq!(schema.schema_type, Some("string".to_string()));
        assert!(schema.format.is_none());
        assert!(schema.reference.is_none());
    }

    #[test]
    fn test_primitive_type_i32() {
        let mut generator = create_generator_from_code("");
        let type_info = TypeInfo::new("i32".to_string());
        let schema = generator.generate_schema(&type_info);

        assert_eq!(schema.schema_type, Some("integer".to_string()));
        assert_eq!(schema.format, Some("int32".to_string()));
        assert!(schema.reference.is_none());
    }

    #[test]
    fn test_primitive_type_i64() {
        let mut generator = create_generator_from_code("");
        let type_info = TypeInfo::new("i64".to_string());
        let schema = generator.generate_schema(&type_info);

        assert_eq!(schema.schema_type, Some("integer".to_string()));
        assert_eq!(schema.format, Some("int64".to_string()));
    }

    #[test]
    fn test_primitive_type_f32() {
        let mut generator = create_generator_from_code("");
        let type_info = TypeInfo::new("f32".to_string());
        let schema = generator.generate_schema(&type_info);

        assert_eq!(schema.schema_type, Some("number".to_string()));
        assert_eq!(schema.format, Some("float".to_string()));
    }

    #[test]
    fn test_primitive_type_f64() {
        let mut generator = create_generator_from_code("");
        let type_info = TypeInfo::new("f64".to_string());
        let schema = generator.generate_schema(&type_info);

        assert_eq!(schema.schema_type, Some("number".to_string()));
        assert_eq!(schema.format, Some("double".to_string()));
    }

    #[test]
    fn test_primitive_type_bool() {
        let mut generator = create_generator_from_code("");
        let type_info = TypeInfo::new("bool".to_string());
        let schema = generator.generate_schema(&type_info);

        assert_eq!(schema.schema_type, Some("boolean".to_string()));
        assert!(schema.format.is_none());
    }

    #[test]
    fn test_vec_type() {
        let mut generator = create_generator_from_code("");
        let inner = TypeInfo::new("String".to_string());
        let type_info = TypeInfo::vec(inner);
        let schema = generator.generate_schema(&type_info);

        assert_eq!(schema.schema_type, Some("array".to_string()));
        assert!(schema.items.is_some());

        let items = schema.items.unwrap();
        assert_eq!(items.schema_type, Some("string".to_string()));
    }

    #[test]
    fn test_option_type() {
        let mut generator = create_generator_from_code("");
        let inner = TypeInfo::new("i32".to_string());
        let type_info = TypeInfo::option(inner);
        let schema = generator.generate_schema(&type_info);

        // Option<T> should unwrap to T's schema
        assert_eq!(schema.schema_type, Some("integer".to_string()));
        assert_eq!(schema.format, Some("int32".to_string()));
    }

    #[test]
    fn test_struct_schema_generation() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub name: String,
                pub active: bool,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        let type_info = TypeInfo::new("User".to_string());
        let schema = generator.generate_schema(&type_info);

        // Should return a reference
        assert!(schema.reference.is_some());
        assert_eq!(
            schema.reference.unwrap(),
            "#/components/schemas/User".to_string()
        );

        // Check that the schema was added to the collection
        let schemas = generator.get_schemas();
        assert!(schemas.contains_key("User"));

        let user_schema = &schemas["User"];
        assert_eq!(user_schema.schema_type, Some("object".to_string()));
        assert!(user_schema.properties.is_some());

        let properties = user_schema.properties.as_ref().unwrap();
        assert_eq!(properties.len(), 3);
        assert!(properties.contains_key("id"));
        assert!(properties.contains_key("name"));
        assert!(properties.contains_key("active"));

        // All fields should be required
        assert!(user_schema.required.is_some());
        let required = user_schema.required.as_ref().unwrap();
        assert_eq!(required.len(), 3);
    }

    #[test]
    fn test_struct_with_optional_field() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub email: Option<String>,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        let type_info = TypeInfo::new("User".to_string());
        generator.generate_schema(&type_info);

        let schemas = generator.get_schemas();
        let user_schema = &schemas["User"];

        // Only id should be required
        assert!(user_schema.required.is_some());
        let required = user_schema.required.as_ref().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "id");
    }

    #[test]
    fn test_struct_with_vec_field() {
        let code = r#"
            pub struct Post {
                pub id: u32,
                pub tags: Vec<String>,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        let type_info = TypeInfo::new("Post".to_string());
        generator.generate_schema(&type_info);

        let schemas = generator.get_schemas();
        let post_schema = &schemas["Post"];

        let properties = post_schema.properties.as_ref().unwrap();
        let tags_property = &properties["tags"];

        assert_eq!(tags_property.property_type, Some("array".to_string()));
        assert!(tags_property.items.is_some());
    }

    #[test]
    fn test_struct_with_serde_rename() {
        let code = r#"
            use serde::{Deserialize, Serialize};
            
            #[derive(Serialize, Deserialize)]
            pub struct User {
                pub id: u32,
                #[serde(rename = "userName")]
                pub name: String,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        let type_info = TypeInfo::new("User".to_string());
        generator.generate_schema(&type_info);

        let schemas = generator.get_schemas();
        let user_schema = &schemas["User"];

        let properties = user_schema.properties.as_ref().unwrap();
        // Should use the renamed field name
        assert!(properties.contains_key("userName"));
        assert!(!properties.contains_key("name"));
    }

    #[test]
    fn test_struct_with_serde_skip() {
        let code = r#"
            use serde::{Deserialize, Serialize};
            
            #[derive(Serialize, Deserialize)]
            pub struct User {
                pub id: u32,
                #[serde(skip)]
                pub password: String,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        let type_info = TypeInfo::new("User".to_string());
        generator.generate_schema(&type_info);

        let schemas = generator.get_schemas();
        let user_schema = &schemas["User"];

        let properties = user_schema.properties.as_ref().unwrap();
        // Skipped field should not be in properties
        assert_eq!(properties.len(), 1);
        assert!(properties.contains_key("id"));
        assert!(!properties.contains_key("password"));
    }

    #[test]
    fn test_enum_schema_generation() {
        let code = r#"
            pub enum Status {
                Active,
                Inactive,
                Pending,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        let type_info = TypeInfo::new("Status".to_string());
        let schema = generator.generate_schema(&type_info);

        // Should return a reference
        assert!(schema.reference.is_some());
        assert_eq!(
            schema.reference.unwrap(),
            "#/components/schemas/Status".to_string()
        );

        // Check the enum schema
        let schemas = generator.get_schemas();
        let status_schema = &schemas["Status"];

        assert_eq!(status_schema.schema_type, Some("string".to_string()));
        assert!(status_schema.enum_values.is_some());

        let variants = status_schema.enum_values.as_ref().unwrap();
        assert_eq!(variants.len(), 3);
        assert!(variants.contains(&"Active".to_string()));
        assert!(variants.contains(&"Inactive".to_string()));
        assert!(variants.contains(&"Pending".to_string()));
    }

    #[test]
    fn test_nested_struct_schema() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub profile: Profile,
            }
            
            pub struct Profile {
                pub bio: String,
                pub avatar: String,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        let type_info = TypeInfo::new("User".to_string());
        generator.generate_schema(&type_info);

        let schemas = generator.get_schemas();
        
        // Both User and Profile should be in schemas
        assert!(schemas.contains_key("User"));
        assert!(schemas.contains_key("Profile"));

        let user_schema = &schemas["User"];
        let properties = user_schema.properties.as_ref().unwrap();
        let profile_property = &properties["profile"];

        // Profile field should be a reference
        assert!(profile_property.reference.is_some());
        assert_eq!(
            profile_property.reference.as_ref().unwrap(),
            "#/components/schemas/Profile"
        );
    }

    #[test]
    fn test_parameter_schema_path() {
        let mut generator = create_generator_from_code("");
        let param = Parameter::new(
            "id".to_string(),
            ParameterLocation::Path,
            TypeInfo::new("u32".to_string()),
            true,
        );

        let param_schema = generator.generate_parameter_schema(&param);

        assert_eq!(param_schema.name, "id");
        assert_eq!(param_schema.location, "path");
        assert!(param_schema.required);
        assert_eq!(param_schema.schema.schema_type, Some("integer".to_string()));
    }

    #[test]
    fn test_parameter_schema_query() {
        let mut generator = create_generator_from_code("");
        let param = Parameter::new(
            "page".to_string(),
            ParameterLocation::Query,
            TypeInfo::new("i32".to_string()),
            false,
        );

        let param_schema = generator.generate_parameter_schema(&param);

        assert_eq!(param_schema.name, "page");
        assert_eq!(param_schema.location, "query");
        assert!(!param_schema.required);
        assert_eq!(param_schema.schema.schema_type, Some("integer".to_string()));
    }

    #[test]
    fn test_parameter_schema_header() {
        let mut generator = create_generator_from_code("");
        let param = Parameter::new(
            "Authorization".to_string(),
            ParameterLocation::Header,
            TypeInfo::new("String".to_string()),
            true,
        );

        let param_schema = generator.generate_parameter_schema(&param);

        assert_eq!(param_schema.name, "Authorization");
        assert_eq!(param_schema.location, "header");
        assert!(param_schema.required);
        assert_eq!(param_schema.schema.schema_type, Some("string".to_string()));
    }

    #[test]
    fn test_complex_nested_type() {
        let code = r#"
            pub struct Response {
                pub data: Option<Vec<User>>,
            }
            
            pub struct User {
                pub id: u32,
                pub name: String,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        let type_info = TypeInfo::new("Response".to_string());
        generator.generate_schema(&type_info);

        let schemas = generator.get_schemas();
        assert!(schemas.contains_key("Response"));
        assert!(schemas.contains_key("User"));

        let response_schema = &schemas["Response"];
        let properties = response_schema.properties.as_ref().unwrap();
        let data_property = &properties["data"];

        // data is Option<Vec<User>>, so it should be an array
        assert_eq!(data_property.property_type, Some("array".to_string()));
        assert!(data_property.items.is_some());

        // The items should reference User
        let items = data_property.items.as_ref().unwrap();
        assert!(items.reference.is_some());
        assert_eq!(
            items.reference.as_ref().unwrap(),
            "#/components/schemas/User"
        );

        // data field should not be required (it's Option)
        let required = response_schema.required.as_ref();
        assert!(required.is_none() || !required.unwrap().contains(&"data".to_string()));
    }

    #[test]
    fn test_unknown_type_fallback() {
        let mut generator = create_generator_from_code("");
        let type_info = TypeInfo::new("UnknownType".to_string());
        let schema = generator.generate_schema(&type_info);

        // Should fallback to object type
        assert_eq!(schema.schema_type, Some("object".to_string()));
        assert!(schema.reference.is_none());
    }

    #[test]
    fn test_schema_caching() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub name: String,
            }
        "#;

        let mut generator = create_generator_from_code(code);
        
        // Generate schema twice
        let type_info = TypeInfo::new("User".to_string());
        generator.generate_schema(&type_info);
        generator.generate_schema(&type_info);

        // Should only have one entry in schemas
        let schemas = generator.get_schemas();
        assert_eq!(schemas.len(), 1);
        assert!(schemas.contains_key("User"));
    }
}
