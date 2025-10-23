use crate::extractor::TypeInfo;
use crate::parser::ParsedFile;
use log::{debug, warn};
use std::collections::{HashMap, HashSet};

/// Type resolver - resolves Rust type definitions to structured type information
pub struct TypeResolver {
    /// All parsed files indexed by their path
    parsed_files: Vec<ParsedFile>,
    /// Cache of resolved types to avoid redundant parsing
    type_cache: HashMap<String, ResolvedType>,
    /// Track types currently being resolved to detect circular references
    resolving_stack: HashSet<String>,
}

/// Resolved type information
#[derive(Debug, Clone)]
pub struct ResolvedType {
    /// The type name
    pub name: String,
    /// The kind of type (struct, enum, primitive, etc.)
    pub kind: TypeKind,
}

/// Type kind - represents different categories of types
#[derive(Debug, Clone)]
pub enum TypeKind {
    /// A struct type with fields
    Struct(StructDef),
    /// An enum type with variants
    Enum(EnumDef),
    /// A primitive type (String, i32, etc.)
    Primitive(PrimitiveType),
    /// A generic type parameter
    Generic(String),
}

/// Struct definition with fields
#[derive(Debug, Clone)]
pub struct StructDef {
    /// The fields of the struct
    pub fields: Vec<FieldDef>,
}

/// Field definition in a struct
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// Field name
    pub name: String,
    /// Type information for the field
    pub type_info: TypeInfo,
    /// Whether the field is optional (wrapped in `Option<T>`)
    pub optional: bool,
    /// Serde attributes applied to this field
    pub serde_attrs: SerdeAttributes,
}

/// Enum definition with variants
#[derive(Debug, Clone)]
pub struct EnumDef {
    /// The variants of the enum
    pub variants: Vec<String>,
}

/// Primitive types supported
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    String,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
}

/// Serde attributes for a field
#[derive(Debug, Clone, Default)]
pub struct SerdeAttributes {
    /// Renamed field name
    pub rename: Option<String>,
    /// Whether to skip this field during serialization
    pub skip: bool,
    /// Whether to flatten this field
    pub flatten: bool,
}

impl TypeResolver {
    /// Create a new TypeResolver with parsed files
    pub fn new(parsed_files: Vec<ParsedFile>) -> Self {
        debug!("Initializing TypeResolver with {} files", parsed_files.len());
        Self {
            parsed_files,
            type_cache: HashMap::new(),
            resolving_stack: HashSet::new(),
        }
    }

    /// Find a struct definition by name across all parsed files
    pub fn find_struct_definition(&self, name: &str) -> Option<&syn::ItemStruct> {
        debug!("Searching for struct definition: {}", name);
        
        for parsed_file in &self.parsed_files {
            for item in &parsed_file.syntax_tree.items {
                if let syn::Item::Struct(item_struct) = item {
                    if item_struct.ident == name {
                        debug!("Found struct {} in {}", name, parsed_file.path.display());
                        return Some(item_struct);
                    }
                }
            }
        }
        
        debug!("Struct {} not found", name);
        None
    }

    /// Find an enum definition by name across all parsed files
    pub fn find_enum_definition(&self, name: &str) -> Option<&syn::ItemEnum> {
        debug!("Searching for enum definition: {}", name);
        
        for parsed_file in &self.parsed_files {
            for item in &parsed_file.syntax_tree.items {
                if let syn::Item::Enum(item_enum) = item {
                    if item_enum.ident == name {
                        debug!("Found enum {} in {}", name, parsed_file.path.display());
                        return Some(item_enum);
                    }
                }
            }
        }
        
        debug!("Enum {} not found", name);
        None
    }

    /// Resolve a type by name
    pub fn resolve_type(&mut self, type_name: &str) -> Option<ResolvedType> {
        debug!("Resolving type: {}", type_name);
        
        // Check cache first
        if let Some(cached) = self.type_cache.get(type_name) {
            debug!("Type {} found in cache", type_name);
            return Some(cached.clone());
        }
        
        // Check for circular reference
        if self.resolving_stack.contains(type_name) {
            warn!("Circular reference detected for type: {}", type_name);
            // Return a placeholder to break the cycle
            let placeholder = ResolvedType {
                name: type_name.to_string(),
                kind: TypeKind::Generic(format!("CircularRef<{}>", type_name)),
            };
            return Some(placeholder);
        }
        
        // Add to resolving stack
        self.resolving_stack.insert(type_name.to_string());
        
        // Check if it's a primitive type
        if let Some(primitive) = Self::parse_primitive_type(type_name) {
            let resolved = ResolvedType {
                name: type_name.to_string(),
                kind: TypeKind::Primitive(primitive),
            };
            self.type_cache.insert(type_name.to_string(), resolved.clone());
            self.resolving_stack.remove(type_name);
            return Some(resolved);
        }
        
        // Try to find struct definition
        let result = if let Some(struct_def) = self.find_struct_definition(type_name) {
            let resolved = self.parse_struct_definition(struct_def);
            self.type_cache.insert(type_name.to_string(), resolved.clone());
            Some(resolved)
        } else if let Some(enum_def) = self.find_enum_definition(type_name) {
            // Try to find enum definition
            let resolved = self.parse_enum_definition(enum_def);
            self.type_cache.insert(type_name.to_string(), resolved.clone());
            Some(resolved)
        } else {
            warn!("Could not resolve type: {}", type_name);
            None
        };
        
        // Remove from resolving stack
        self.resolving_stack.remove(type_name);
        
        result
    }

    /// Recursively resolve nested types in a struct
    pub fn resolve_nested_types(&mut self, type_info: &TypeInfo) {
        debug!("Resolving nested types for: {}", type_info.name);
        
        // Resolve the main type if it's not a primitive
        if Self::parse_primitive_type(&type_info.name).is_none() {
            self.resolve_type(&type_info.name);
        }
        
        // Recursively resolve generic arguments
        for generic_arg in &type_info.generic_args {
            self.resolve_nested_types(generic_arg);
        }
    }

    /// Parse a struct definition into a ResolvedType
    fn parse_struct_definition(&self, item_struct: &syn::ItemStruct) -> ResolvedType {
        let struct_name = item_struct.ident.to_string();
        debug!("Parsing struct definition: {}", struct_name);
        
        let fields = self.parse_struct_fields(item_struct);
        
        ResolvedType {
            name: struct_name,
            kind: TypeKind::Struct(StructDef { fields }),
        }
    }

    /// Parse an enum definition into a ResolvedType
    fn parse_enum_definition(&self, item_enum: &syn::ItemEnum) -> ResolvedType {
        let enum_name = item_enum.ident.to_string();
        debug!("Parsing enum definition: {}", enum_name);
        
        let variants: Vec<String> = item_enum
            .variants
            .iter()
            .map(|v| v.ident.to_string())
            .collect();
        
        debug!("Parsed {} variants", variants.len());
        
        ResolvedType {
            name: enum_name,
            kind: TypeKind::Enum(EnumDef { variants }),
        }
    }

    /// Parse struct fields
    fn parse_struct_fields(&self, item_struct: &syn::ItemStruct) -> Vec<FieldDef> {
        let mut fields = Vec::new();
        
        if let syn::Fields::Named(named_fields) = &item_struct.fields {
            for field in &named_fields.named {
                if let Some(field_def) = self.parse_field(field) {
                    fields.push(field_def);
                }
            }
        }
        
        debug!("Parsed {} fields", fields.len());
        fields
    }

    /// Parse a single field
    fn parse_field(&self, field: &syn::Field) -> Option<FieldDef> {
        let field_name = field.ident.as_ref()?.to_string();
        debug!("Parsing field: {}", field_name);
        
        let type_info = Self::extract_type_info(&field.ty);
        let optional = type_info.is_option;
        let serde_attrs = Self::parse_serde_attributes(&field.attrs);
        
        Some(FieldDef {
            name: field_name,
            type_info,
            optional,
            serde_attrs,
        })
    }

    /// Parse Serde attributes from field attributes
    fn parse_serde_attributes(attrs: &[syn::Attribute]) -> SerdeAttributes {
        let mut serde_attrs = SerdeAttributes::default();
        
        for attr in attrs {
            // Check if this is a serde attribute
            if !attr.path().is_ident("serde") {
                continue;
            }
            
            // Parse the attribute arguments
            if let Ok(meta_list) = attr.meta.require_list() {
                // Convert the entire token stream to a string for parsing
                let tokens_str = meta_list.tokens.to_string();
                
                // Parse rename attribute: #[serde(rename = "...")]
                if let Some(value) = Self::extract_rename_value(&tokens_str) {
                    debug!("Found serde rename: {}", value);
                    serde_attrs.rename = Some(value);
                }
                
                // Parse skip attribute: #[serde(skip)]
                if tokens_str.contains("skip") && !tokens_str.contains("skip_serializing_if") {
                    debug!("Found serde skip");
                    serde_attrs.skip = true;
                }
                
                // Parse flatten attribute: #[serde(flatten)]
                if tokens_str.contains("flatten") {
                    debug!("Found serde flatten");
                    serde_attrs.flatten = true;
                }
            }
        }
        
        serde_attrs
    }

    /// Extract rename value from serde attribute tokens
    fn extract_rename_value(tokens_str: &str) -> Option<String> {
        // Look for pattern: rename = "value"
        if let Some(rename_pos) = tokens_str.find("rename") {
            let after_rename = &tokens_str[rename_pos..];
            if let Some(eq_pos) = after_rename.find('=') {
                let after_eq = &after_rename[eq_pos + 1..];
                // Find the string literal
                if let Some(start_quote) = after_eq.find('"') {
                    let after_start = &after_eq[start_quote + 1..];
                    if let Some(end_quote) = after_start.find('"') {
                        let value = &after_start[..end_quote];
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extract TypeInfo from a syn::Type
    fn extract_type_info(ty: &syn::Type) -> TypeInfo {
        match ty {
            syn::Type::Path(type_path) => {
                Self::extract_type_info_from_path(&type_path.path)
            }
            _ => {
                // For other types, use a generic placeholder
                TypeInfo::new("Unknown".to_string())
            }
        }
    }

    /// Extract TypeInfo from a syn::Path
    fn extract_type_info_from_path(path: &syn::Path) -> TypeInfo {
        if let Some(segment) = path.segments.last() {
            let type_name = segment.ident.to_string();
            
            // Check for Option<T>
            if type_name == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        let inner_type_info = Self::extract_type_info(inner_ty);
                        return TypeInfo::option(inner_type_info);
                    }
                }
            }
            
            // Check for Vec<T>
            if type_name == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        let inner_type_info = Self::extract_type_info(inner_ty);
                        return TypeInfo::vec(inner_type_info);
                    }
                }
            }
            
            // Handle generic types
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                let mut generic_args = Vec::new();
                for arg in &args.args {
                    if let syn::GenericArgument::Type(inner_ty) = arg {
                        generic_args.push(Self::extract_type_info(inner_ty));
                    }
                }
                
                return TypeInfo {
                    name: type_name,
                    is_generic: !generic_args.is_empty(),
                    generic_args,
                    is_option: false,
                    is_vec: false,
                };
            }
            
            // Simple type
            TypeInfo::new(type_name)
        } else {
            TypeInfo::new("Unknown".to_string())
        }
    }

    /// Parse a primitive type name
    fn parse_primitive_type(type_name: &str) -> Option<PrimitiveType> {
        match type_name {
            "String" | "str" => Some(PrimitiveType::String),
            "i8" => Some(PrimitiveType::I8),
            "i16" => Some(PrimitiveType::I16),
            "i32" => Some(PrimitiveType::I32),
            "i64" => Some(PrimitiveType::I64),
            "i128" => Some(PrimitiveType::I128),
            "u8" => Some(PrimitiveType::U8),
            "u16" => Some(PrimitiveType::U16),
            "u32" => Some(PrimitiveType::U32),
            "u64" => Some(PrimitiveType::U64),
            "u128" => Some(PrimitiveType::U128),
            "f32" => Some(PrimitiveType::F32),
            "f64" => Some(PrimitiveType::F64),
            "bool" => Some(PrimitiveType::Bool),
            "char" => Some(PrimitiveType::Char),
            _ => None,
        }
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

    /// Helper function to parse files and create a TypeResolver
    fn create_resolver_from_code(code: &str) -> TypeResolver {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "test.rs", code);
        let parsed = AstParser::parse_file(&file_path).unwrap();
        TypeResolver::new(vec![parsed])
    }

    #[test]
    fn test_resolve_primitive_types() {
        let resolver = create_resolver_from_code("");
        
        let mut resolver = resolver;
        
        // Test various primitive types
        let primitives = vec![
            ("String", PrimitiveType::String),
            ("i32", PrimitiveType::I32),
            ("u64", PrimitiveType::U64),
            ("f32", PrimitiveType::F32),
            ("bool", PrimitiveType::Bool),
        ];
        
        for (type_name, expected_primitive) in primitives {
            let resolved = resolver.resolve_type(type_name);
            assert!(resolved.is_some());
            
            let resolved = resolved.unwrap();
            assert_eq!(resolved.name, type_name);
            
            if let TypeKind::Primitive(prim) = resolved.kind {
                assert_eq!(prim, expected_primitive);
            } else {
                panic!("Expected primitive type for {}", type_name);
            }
        }
    }

    #[test]
    fn test_resolve_simple_struct() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub name: String,
                pub active: bool,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("User");
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        assert_eq!(resolved.name, "User");
        
        if let TypeKind::Struct(struct_def) = resolved.kind {
            assert_eq!(struct_def.fields.len(), 3);
            
            // Check field names
            assert_eq!(struct_def.fields[0].name, "id");
            assert_eq!(struct_def.fields[1].name, "name");
            assert_eq!(struct_def.fields[2].name, "active");
            
            // Check field types
            assert_eq!(struct_def.fields[0].type_info.name, "u32");
            assert_eq!(struct_def.fields[1].type_info.name, "String");
            assert_eq!(struct_def.fields[2].type_info.name, "bool");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_resolve_struct_with_option() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub email: Option<String>,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("User");
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        
        if let TypeKind::Struct(struct_def) = resolved.kind {
            assert_eq!(struct_def.fields.len(), 2);
            
            // Check the Option field
            let email_field = &struct_def.fields[1];
            assert_eq!(email_field.name, "email");
            assert!(email_field.type_info.is_option);
            assert!(email_field.optional);
            assert_eq!(email_field.type_info.name, "String");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_resolve_struct_with_vec() {
        let code = r#"
            pub struct Post {
                pub id: u32,
                pub tags: Vec<String>,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("Post");
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        
        if let TypeKind::Struct(struct_def) = resolved.kind {
            assert_eq!(struct_def.fields.len(), 2);
            
            // Check the Vec field
            let tags_field = &struct_def.fields[1];
            assert_eq!(tags_field.name, "tags");
            assert!(tags_field.type_info.is_vec);
            assert_eq!(tags_field.type_info.name, "String");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_serde_rename() {
        let code = r#"
            use serde::{Deserialize, Serialize};
            
            #[derive(Serialize, Deserialize)]
            pub struct User {
                pub id: u32,
                #[serde(rename = "userName")]
                pub name: String,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("User");
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        
        if let TypeKind::Struct(struct_def) = resolved.kind {
            let name_field = &struct_def.fields[1];
            assert_eq!(name_field.name, "name");
            assert_eq!(name_field.serde_attrs.rename, Some("userName".to_string()));
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_serde_skip() {
        let code = r#"
            use serde::{Deserialize, Serialize};
            
            #[derive(Serialize, Deserialize)]
            pub struct User {
                pub id: u32,
                #[serde(skip)]
                pub password: String,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("User");
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        
        if let TypeKind::Struct(struct_def) = resolved.kind {
            let password_field = &struct_def.fields[1];
            assert_eq!(password_field.name, "password");
            assert!(password_field.serde_attrs.skip);
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_parse_serde_flatten() {
        let code = r#"
            use serde::{Deserialize, Serialize};
            
            #[derive(Serialize, Deserialize)]
            pub struct User {
                pub id: u32,
                #[serde(flatten)]
                pub metadata: Metadata,
            }
            
            pub struct Metadata {
                pub created_at: String,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("User");
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        
        if let TypeKind::Struct(struct_def) = resolved.kind {
            let metadata_field = &struct_def.fields[1];
            assert_eq!(metadata_field.name, "metadata");
            assert!(metadata_field.serde_attrs.flatten);
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_resolve_nested_struct() {
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
        
        let mut resolver = create_resolver_from_code(code);
        
        // Resolve the User struct
        let user_resolved = resolver.resolve_type("User");
        assert!(user_resolved.is_some());
        
        // Resolve the nested Profile struct
        let profile_resolved = resolver.resolve_type("Profile");
        assert!(profile_resolved.is_some());
        
        let profile_resolved = profile_resolved.unwrap();
        if let TypeKind::Struct(struct_def) = profile_resolved.kind {
            assert_eq!(struct_def.fields.len(), 2);
            assert_eq!(struct_def.fields[0].name, "bio");
            assert_eq!(struct_def.fields[1].name, "avatar");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_resolve_enum() {
        let code = r#"
            pub enum Status {
                Active,
                Inactive,
                Pending,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("Status");
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        assert_eq!(resolved.name, "Status");
        
        if let TypeKind::Enum(enum_def) = resolved.kind {
            assert_eq!(enum_def.variants.len(), 3);
            assert_eq!(enum_def.variants[0], "Active");
            assert_eq!(enum_def.variants[1], "Inactive");
            assert_eq!(enum_def.variants[2], "Pending");
        } else {
            panic!("Expected enum type");
        }
    }

    #[test]
    fn test_type_caching() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub name: String,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        
        // Resolve the same type twice
        let resolved1 = resolver.resolve_type("User");
        let resolved2 = resolver.resolve_type("User");
        
        assert!(resolved1.is_some());
        assert!(resolved2.is_some());
        
        // Both should have the same data
        let r1 = resolved1.unwrap();
        let r2 = resolved2.unwrap();
        assert_eq!(r1.name, r2.name);
    }

    #[test]
    fn test_circular_reference_detection() {
        let code = r#"
            pub struct Node {
                pub value: i32,
                pub next: Option<Box<Node>>,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        
        // This should not cause infinite recursion
        let resolved = resolver.resolve_type("Node");
        assert!(resolved.is_some());
    }

    #[test]
    fn test_resolve_nonexistent_type() {
        let code = r#"
            pub struct User {
                pub id: u32,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("NonExistent");
        
        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_nested_types_recursively() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub posts: Vec<Post>,
            }
            
            pub struct Post {
                pub id: u32,
                pub title: String,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        
        // Resolve User
        let user_resolved = resolver.resolve_type("User");
        assert!(user_resolved.is_some());
        
        // Get the TypeInfo for the posts field
        if let Some(user) = user_resolved {
            if let TypeKind::Struct(struct_def) = user.kind {
                let posts_field = &struct_def.fields[1];
                
                // Recursively resolve nested types
                resolver.resolve_nested_types(&posts_field.type_info);
                
                // Post should now be in the cache
                let post_resolved = resolver.resolve_type("Post");
                assert!(post_resolved.is_some());
            }
        }
    }

    #[test]
    fn test_complex_generic_types() {
        let code = r#"
            pub struct Response {
                pub data: Option<Vec<String>>,
            }
        "#;
        
        let mut resolver = create_resolver_from_code(code);
        let resolved = resolver.resolve_type("Response");
        
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        
        if let TypeKind::Struct(struct_def) = resolved.kind {
            let data_field = &struct_def.fields[0];
            assert_eq!(data_field.name, "data");
            assert!(data_field.type_info.is_option);
            
            // The inner type should be Vec<String>
            if let Some(inner) = data_field.type_info.generic_args.first() {
                assert!(inner.is_vec);
                assert_eq!(inner.name, "String");
            } else {
                panic!("Expected generic args for Option");
            }
        } else {
            panic!("Expected struct type");
        }
    }
}
