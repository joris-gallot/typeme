use openapiv3::{IntegerType, NumberType, ReferenceOr, Schema, SchemaKind, StringType, Type};

#[derive(Debug, Clone)]
enum ObjectOrPrimitiveOrRef {
    TypeObject(TypeObject),
    PrimitiveProperty(PrimitiveProperty),
    RefProperty(RefProperty),
}

#[derive(Debug, Clone)]
enum UnionOrIntersection {
    Union,
    Intersection,
}

#[derive(Debug)]
pub struct TypeInterface {
    name: String,
    expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
struct TypeObject {
    properties: Vec<ObjectProperty>,
    is_array: bool,
}

#[derive(Debug, Clone)]
enum PrimitiveType {
    String,
    Number,
    Boolean,
    Null,
    Any,
}

#[derive(Debug, Clone)]
struct RefProperty {
    reference: String,
    is_array: bool,
}

#[derive(Debug, Clone)]
struct PrimitiveProperty {
    primitive_type: PrimitiveType,
    enumeration: Vec<String>,
    is_array: bool,
}

#[derive(Debug, Clone)]
struct ObjectProperty {
    name: String,
    expressions: Vec<Expression>,
    required: bool,
}

#[derive(Debug, Clone)]
struct Expression {
    types: Vec<ObjectOrPrimitiveOrRef>,
    link: Option<UnionOrIntersection>,
}

impl TypeInterface {
    fn get_separator(separator: &Option<UnionOrIntersection>) -> &'static str {
        match separator {
            Some(UnionOrIntersection::Union) => " | ",
            Some(UnionOrIntersection::Intersection) => " & ",
            None => " | ",
        }
    }

    fn reference_to_string(reference: &RefProperty) -> String {
        reference
            .is_array
            .then_some(format!("{}[]", reference.reference))
            .unwrap_or(reference.reference.to_string())
    }

    fn primitive_to_string(primitive: &PrimitiveProperty, expression_is_array: bool) -> String {
        match primitive.primitive_type {
            PrimitiveType::String => {
                if primitive.enumeration.is_empty() {
                    if expression_is_array {
                        "string".to_string()
                    } else {
                        primitive
                            .is_array
                            .then_some("string[]".to_string())
                            .unwrap_or("string".to_string())
                    }
                } else {
                    format!(
                        "{}",
                        primitive
                            .enumeration
                            .iter()
                            .map(|s| format!("\"{}\"", s))
                            .collect::<Vec<String>>()
                            .join(TypeInterface::get_separator(&Some(
                                UnionOrIntersection::Union
                            )))
                    )
                }
            }
            PrimitiveType::Number => {
                if primitive.enumeration.is_empty() {
                    if expression_is_array {
                        "number".to_string()
                    } else {
                        primitive
                            .is_array
                            .then_some("number[]".to_string())
                            .unwrap_or("number".to_string())
                    }
                } else {
                    format!(
                        "{}",
                        primitive
                            .enumeration
                            .join(TypeInterface::get_separator(&Some(
                                UnionOrIntersection::Union
                            )))
                    )
                }
            }
            PrimitiveType::Boolean => {
                if expression_is_array {
                    "boolean".to_string()
                } else {
                    primitive
                        .is_array
                        .then_some("boolean[]".to_string())
                        .unwrap_or("boolean".to_string())
                }
            }
            PrimitiveType::Null => {
                if expression_is_array {
                    "null".to_string()
                } else {
                    primitive
                        .is_array
                        .then_some("null[]".to_string())
                        .unwrap_or("null".to_string())
                }
            }
            PrimitiveType::Any => {
                if expression_is_array {
                    "any".to_string()
                } else {
                    primitive
                        .is_array
                        .then_some("any[]".to_string())
                        .unwrap_or("any".to_string())
                }
            }
        }
    }

    fn format_string_expression(exp_string: String, need_array: bool) -> String {
        format!(
            "{}{}{}{}",
            if need_array { "(" } else { "" },
            exp_string,
            if need_array { ")" } else { "" },
            if need_array { "[]" } else { "" }
        )
    }

    fn expression_is_array(expression: &Expression) -> bool {
        expression.types.len() > 1
            && expression.types.iter().all(|t| match t {
                ObjectOrPrimitiveOrRef::TypeObject(obj) => obj.is_array,
                ObjectOrPrimitiveOrRef::PrimitiveProperty(primitive) => primitive.is_array,
                ObjectOrPrimitiveOrRef::RefProperty(reference) => reference.is_array,
            })
    }

    fn type_object_to_string(
        object: &ObjectOrPrimitiveOrRef,
        depth: usize,
        expression_is_array: bool,
    ) -> String {
        match object {
            ObjectOrPrimitiveOrRef::TypeObject(type_object) => {
                if type_object.properties.is_empty() {
                    return "{}".to_string();
                }

                let mut object_string = Vec::new();

                for property in &type_object.properties {
                    let ts_types_string = property
                        .expressions
                        .iter()
                        .map(|expression| {
                            let expression_need_array =
                                TypeInterface::expression_is_array(expression);

                            let exp_string = expression
                                .types
                                .iter()
                                .map(|t| match t {
                                    ObjectOrPrimitiveOrRef::TypeObject(obj) => {
                                        TypeInterface::type_object_to_string(
                                            &ObjectOrPrimitiveOrRef::TypeObject(obj.clone()),
                                            depth + 1,
                                            expression_need_array,
                                        )
                                    }
                                    ObjectOrPrimitiveOrRef::PrimitiveProperty(primitive) => {
                                        TypeInterface::primitive_to_string(
                                            primitive,
                                            expression_need_array,
                                        )
                                    }
                                    ObjectOrPrimitiveOrRef::RefProperty(reference) => {
                                        TypeInterface::reference_to_string(reference)
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join(TypeInterface::get_separator(&expression.link));

                            TypeInterface::format_string_expression(
                                exp_string,
                                expression_need_array,
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(TypeInterface::get_separator(&Some(
                            UnionOrIntersection::Union,
                        )));

                    object_string.push(format!(
                        "{}{}{}: {};",
                        "  ".repeat(depth),
                        property.name,
                        if property.required { "" } else { "?" },
                        ts_types_string,
                    ));
                }

                format!(
                    "{{\n{}\n{}}}{}",
                    object_string.join("\n"),
                    "  ".repeat(depth - 1),
                    if type_object.is_array && !expression_is_array {
                        "[]"
                    } else {
                        ""
                    }
                )
            }
            ObjectOrPrimitiveOrRef::PrimitiveProperty(primitive) => {
                TypeInterface::primitive_to_string(primitive, expression_is_array)
            }
            ObjectOrPrimitiveOrRef::RefProperty(reference) => {
                TypeInterface::reference_to_string(reference)
            }
        }
    }

    pub fn to_string(&self) -> String {
        if self.expressions.is_empty() {
            return String::new();
        }

        let types = self
            .expressions
            .iter()
            .map(|expression| {
                let expression_is_array = TypeInterface::expression_is_array(expression);
                let exp_string = expression
                    .types
                    .iter()
                    .map(|t| TypeInterface::type_object_to_string(t, 1, expression_is_array))
                    .collect::<Vec<String>>()
                    .join(TypeInterface::get_separator(&expression.link));

                TypeInterface::format_string_expression(exp_string, expression_is_array)
            })
            .collect::<Vec<String>>();

        format!(
            "type {} = {};",
            self.name,
            types.join(TypeInterface::get_separator(&Some(
                UnionOrIntersection::Union
            )))
        )
    }
}

trait HasEnumeration {
    type ReturnType;
    fn get_enumeration(&self) -> &Vec<Option<Self::ReturnType>>;
    fn to_string(&self, value: &Self::ReturnType) -> String;
}

impl HasEnumeration for NumberType {
    type ReturnType = f64;
    fn get_enumeration(&self) -> &Vec<Option<Self::ReturnType>> {
        &self.enumeration
    }
    fn to_string(&self, value: &Self::ReturnType) -> String {
        value.to_string()
    }
}

impl HasEnumeration for IntegerType {
    type ReturnType = i64;
    fn get_enumeration(&self) -> &Vec<Option<Self::ReturnType>> {
        &self.enumeration
    }
    fn to_string(&self, value: &Self::ReturnType) -> String {
        value.to_string()
    }
}

impl HasEnumeration for StringType {
    type ReturnType = String;
    fn get_enumeration(&self) -> &Vec<Option<Self::ReturnType>> {
        &self.enumeration
    }
    fn to_string(&self, value: &Self::ReturnType) -> String {
        value.to_string()
    }
}

fn get_string_number_expression<T>(
    type_with_enum: &T,
    primitive_type: PrimitiveType,
    is_array: bool,
) -> Expression
where
    T: HasEnumeration,
{
    let enumeration = type_with_enum
        .get_enumeration()
        .iter()
        .filter(|s| s.is_some())
        .map(|s| T::to_string(type_with_enum, s.as_ref().unwrap()))
        .collect::<Vec<String>>();

    Expression {
        types: vec![ObjectOrPrimitiveOrRef::PrimitiveProperty(
            PrimitiveProperty {
                primitive_type: primitive_type,
                enumeration: enumeration,
                is_array: is_array,
            },
        )],
        link: None,
    }
}

fn schema_to_typescript_any_one_all_of_types(
    schema: &Vec<ReferenceOr<Schema>>,
    is_array: bool,
    separator: Option<UnionOrIntersection>,
) -> Vec<ObjectOrPrimitiveOrRef> {
    schema
        .iter()
        .map(|any_of_item| {
            schema_to_typescript_expressions(any_of_item, is_array, separator.clone())
        })
        .flatten()
        .map(|expression| expression.types)
        .flatten()
        .collect()
}

trait SchemaLike {
    fn as_schema(&self) -> &Schema;
}

impl SchemaLike for Schema {
    fn as_schema(&self) -> &Schema {
        self
    }
}

impl SchemaLike for Box<Schema> {
    fn as_schema(&self) -> &Schema {
        self.as_ref()
    }
}

fn schema_to_typescript_expressions<T: SchemaLike>(
    schema: &ReferenceOr<T>,
    is_array: bool,
    separator: Option<UnionOrIntersection>,
) -> Vec<Expression> {
    match schema {
        ReferenceOr::Item(schema) => {
            let mut expressions: Vec<Expression> = Vec::new();
            let schema = schema.as_schema();

            match &schema.schema_kind {
                SchemaKind::Type(Type::String(string_type)) => {
                    expressions.push(get_string_number_expression(
                        string_type,
                        PrimitiveType::String,
                        is_array,
                    ));
                }
                SchemaKind::Type(Type::Number(number_type)) => {
                    expressions.push(get_string_number_expression(
                        number_type,
                        PrimitiveType::Number,
                        is_array,
                    ));
                }
                SchemaKind::Type(Type::Integer(integer_type)) => {
                    expressions.push(get_string_number_expression(
                        integer_type,
                        PrimitiveType::Number,
                        is_array,
                    ));
                }
                SchemaKind::Type(Type::Boolean(_)) => {
                    expressions.push(Expression {
                        types: vec![ObjectOrPrimitiveOrRef::PrimitiveProperty(
                            PrimitiveProperty {
                                primitive_type: PrimitiveType::Boolean,
                                enumeration: vec![],
                                is_array: is_array,
                            },
                        )],
                        link: None,
                    });
                }
                SchemaKind::Type(Type::Array(v)) => {
                    let array_expressions: Vec<Expression> = match &v.items {
                        Some(item) => {
                            schema_to_typescript_expressions(item, true, separator.clone())
                        }
                        None => vec![Expression {
                            types: vec![ObjectOrPrimitiveOrRef::PrimitiveProperty(
                                PrimitiveProperty {
                                    primitive_type: PrimitiveType::Any,
                                    enumeration: vec![],
                                    is_array: true,
                                },
                            )],
                            link: None,
                        }],
                    };

                    expressions.extend(array_expressions);
                }
                SchemaKind::Type(Type::Object(object)) => {
                    let properties: Vec<ObjectProperty> = object
                        .properties
                        .iter()
                        .map(|(key, value)| ObjectProperty {
                            name: key.to_string(),
                            expressions: schema_to_typescript_expressions(value, false, None),
                            required: object.required.contains(key),
                        })
                        .collect();

                    expressions.push(Expression {
                        types: vec![ObjectOrPrimitiveOrRef::TypeObject(TypeObject {
                            properties,
                            is_array: is_array,
                        })],
                        link: None,
                    });
                }
                SchemaKind::AnyOf { any_of } => {
                    expressions.push(Expression {
                        types: schema_to_typescript_any_one_all_of_types(any_of, is_array, None),
                        link: Some(UnionOrIntersection::Union),
                    });
                }
                SchemaKind::OneOf { one_of } => {
                    expressions.push(Expression {
                        types: schema_to_typescript_any_one_all_of_types(one_of, is_array, None),
                        link: Some(UnionOrIntersection::Union),
                    });
                }
                SchemaKind::AllOf { all_of } => {
                    expressions.push(Expression {
                        types: schema_to_typescript_any_one_all_of_types(all_of, is_array, None),
                        link: Some(UnionOrIntersection::Intersection),
                    });
                }
                _ => {
                    println!("unknown schema kind for {:?}", schema);
                }
            }

            if schema.schema_data.nullable {
                expressions.push(Expression {
                    types: vec![ObjectOrPrimitiveOrRef::PrimitiveProperty(
                        PrimitiveProperty {
                            primitive_type: PrimitiveType::Null,
                            enumeration: vec![],
                            is_array: false,
                        },
                    )],
                    link: None,
                });
            }

            return expressions;
        }
        ReferenceOr::Reference { reference } => {
            let reference_name = reference.split('/').last().unwrap_or_default().to_string();
            return vec![Expression {
                types: vec![ObjectOrPrimitiveOrRef::RefProperty(RefProperty {
                    reference: reference_name,
                    is_array: is_array,
                })],
                link: separator,
            }];
        }
    }
}

pub fn schema_to_typescript(name: String, schema: ReferenceOr<Schema>) -> TypeInterface {
    TypeInterface {
        name: name,
        expressions: schema_to_typescript_expressions(&schema, false, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_object() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "metadata": {
                    "type": "object",
                    "properties": {}
                }
            }
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("EmptyObject".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type EmptyObject = {
  metadata?: {};
};"##;
        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_basic_object() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "title": { "type": "string" },
                "author": { "type": "string" },
                "publishedDate": { "type": "string", "format": "date" },
                "rating": { "type": "number", "format": "float" },
                "age": { "type": "integer" }
            }
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface = schema_to_typescript("Book".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type Book = {
  id?: string;
  title?: string;
  author?: string;
  publishedDate?: string;
  rating?: number;
  age?: number;
};"##;
        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_array() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "genres": { "type": "array", "items": { "type": "string" } },
                "tags": { "type": "array", "items": { "type": "string" } }
            }
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("BookMetadata".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type BookMetadata = {
  id?: string;
  genres?: string[];
  tags?: string[];
};"##;
        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_required_properties() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "title": { "type": "string" },
                "author": { "type": "string" },
                "genres": { "type": "array", "items": { "type": "string" } },
                "publishedDate": { "type": "string", "format": "date" },
                "rating": { "type": "number", "format": "float" }
            },
            "required": ["title", "author"]
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface = schema_to_typescript("NewBook".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type NewBook = {
  title: string;
  author: string;
  genres?: string[];
  publishedDate?: string;
  rating?: number;
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_nullable_properties() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "reviewer": {
                    "type": "string",
                    "description": "Name of the reviewer"
                },
                "comment": {
                    "type": "string",
                    "nullable": true,
                    "description": "Review comment"
                },
                "rating": {
                    "type": "number",
                    "format": "float",
                    "nullable": true,
                    "description": "Rating given by the reviewer"
                },
                "date": {
                    "type": "string",
                    "format": "date-time",
                    "nullable": true,
                    "description": "Date of the review"
                }
            }
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface = schema_to_typescript("Review".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type Review = {
  reviewer?: string;
  comment?: string | null;
  rating?: number | null;
  date?: string | null;
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_string_enum() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "status": {
                    "type": "string",
                    "enum": ["draft", "published", "archived"]
                },
                "visibility": {
                    "type": "string",
                    "enum": ["public", "private"],
                    "nullable": true
                }
            },
            "required": ["id", "status"]
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface = schema_to_typescript("Post".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type Post = {
  id: string;
  status: "draft" | "published" | "archived";
  visibility?: "public" | "private" | null;
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_number_enum() {
        let schema_json = r#"
        {
            "type": "object", 
            "properties": {
                "id": { "type": "string" },
                "priority": {
                    "type": "number",
                    "enum": [1.5, 2.5, 3.5]
                },
                "score": {
                    "type": "number",
                    "enum": [0.5, 1.0],
                    "nullable": true
                }
            },
            "required": ["id", "priority"]
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface = schema_to_typescript("Task".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type Task = {
  id: string;
  priority: 1.5 | 2.5 | 3.5;
  score?: 0.5 | 1 | null;
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_integer_enum() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "level": {
                    "type": "integer",
                    "enum": [1, 2, 3]
                },
                "grade": {
                    "type": "integer", 
                    "enum": [0, 1],
                    "nullable": true
                }
            },
            "required": ["id", "level"]
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface = schema_to_typescript("Grade".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type Grade = {
  id: string;
  level: 1 | 2 | 3;
  grade?: 0 | 1 | null;
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_oneof() {
        let schema_json = r##"
        {
            "oneOf": [
                { "$ref": "#/components/schemas/Book" },
                {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "genres": { "type": "array", "items": { "type": "string" } },
                        "rating": { "type": "number", "format": "float" }
                    }
                }
            ]
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("SearchCriteria".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type SearchCriteria = Book | {
  query?: string;
  genres?: string[];
  rating?: number;
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_allof() {
        let schema_json = r##"
        {
            "allOf": [
                { "$ref": "#/components/schemas/Book" },
                {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "genres": { "type": "array", "items": { "type": "string" } },
                        "rating": { "type": "number", "format": "float" }
                    }
                }
            ]
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("BookWithMetadata".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type BookWithMetadata = Book & {
  query?: string;
  genres?: string[];
  rating?: number;
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_anyof() {
        let schema_json = r##"
        {
            "anyOf": [
                {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "age": { "type": "number" }
                    },
                    "required": ["name"]
                },
                {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "role": {
                            "type": "string",
                            "enum": ["admin", "user"]
                        }
                    },
                    "required": ["id", "role"]
                }
            ]
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("UserInfo".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type UserInfo = {
  name: string;
  age?: number;
} | {
  id: string;
  role: "admin" | "user";
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_array_with_oneof() {
        let schema_json = r##"
        {
            "type": "array",
            "items": {
                "oneOf": [
                    { "type": "string" },
                    { "type": "number" },
                    {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "value": { "type": "number" }
                        },
                        "required": ["name", "value"]
                    }
                ]
            }
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("MixedArray".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type MixedArray = (string | number | {
  name: string;
  value: number;
})[];"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_array_with_allof() {
        let schema_json = r##"
        {
            "type": "array",
            "items": {
                "allOf": [
                    { "type": "object",
                      "properties": {
                          "id": { "type": "string" },
                          "name": { "type": "string" }
                      },
                      "required": ["id"]
                    },
                    {
                        "type": "object",
                        "properties": {
                            "metadata": {
                                "type": "object",
                                "properties": {
                                    "created": { "type": "string" },
                                    "modified": { "type": "string" }
                                }
                            }
                        }
                    }
                ]
            }
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("CombinedArray".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type CombinedArray = ({
  id: string;
  name?: string;
} & {
  metadata?: {
    created?: string;
    modified?: string;
  };
})[];"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_array_with_anyof() {
        let schema_json = r##"
        {
            "type": "array",
            "items": {
                "anyOf": [
                    { "type": "string" },
                    { "type": "number" },
                    {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "value": { "type": "number" }
                        },
                        "required": ["name", "value"]
                    }
                ]
            }
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("MixedAnyArray".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type MixedAnyArray = (string | number | {
  name: string;
  value: number;
})[];"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_anyof_array_primitive_object() {
        let schema_json = r##"
        {
            "type": "object", 
            "properties": {
                "key": {
                    "type": "string"
                },
                "value": {
                    "anyOf": [
                        {
                            "type": "array",
                            "items": {
                                "type": "number"
                            }
                        },
                        {
                            "type": "string"
                        },
                        {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string" },
                                    "count": { "type": "number" }
                                },
                                "required": ["name", "count"]
                            }
                        },
                        {
                            "type": "array",
                            "items": {
                                "$ref": "#/components/schemas/SomeType"
                            }
                        }
                    ]
                }
            },
            "required": [
                "key",
                "value"
            ]
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("MixedValue".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type MixedValue = {
  key: string;
  value: number[] | string | {
    name: string;
    count: number;
  }[] | SomeType[];
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_oneof_array_primitive_object() {
        let schema_json = r##"
        {
            "type": "object", 
            "properties": {
                "key": {
                    "type": "string"
                },
                "value": {
                    "oneOf": [
                        {
                            "type": "array",
                            "items": {
                                "type": "number"
                            }
                        },
                        {
                            "type": "string"
                        },
                        {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string" },
                                    "count": { "type": "number" }
                                },
                                "required": ["name", "count"]
                            }
                        },
                        {
                            "type": "array",
                            "items": {
                                "$ref": "#/components/schemas/SomeType"
                            }
                        }
                    ]
                }
            },
            "required": [
                "key",
                "value"
            ]
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("MixedValue".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type MixedValue = {
  key: string;
  value: number[] | string | {
    name: string;
    count: number;
  }[] | SomeType[];
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_nested_objects() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "name": { "type": "string" },
                "address": {
                    "type": "object",
                    "properties": {
                        "street": { "type": "string" },
                        "city": { "type": "string" },
                        "country": { "type": "string" },
                        "coordinates": {
                            "type": "object",
                            "properties": {
                                "latitude": { "type": "number" },
                                "longitude": { "type": "number" }
                            },
                            "required": ["latitude", "longitude"]
                        }
                    },
                    "required": ["street", "city"]
                }
            },
            "required": ["id", "name", "address"]
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("Location".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type Location = {
  id: string;
  name: string;
  address: {
    street: string;
    city: string;
    country?: string;
    coordinates?: {
      latitude: number;
      longitude: number;
    };
  };
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_nested_arrays() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "categories": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "subcategories": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        },
                        "required": ["name"]
                    }
                }
            },
            "required": ["id"]
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface = schema_to_typescript("Product".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type Product = {
  id: string;
  categories?: {
    name: string;
    subcategories?: string[];
  }[];
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_complex_nested_arrays() {
        let schema_json = r#"
        {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "departments": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "teams": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "teamName": { "type": "string" },
                                        "members": {
                                            "type": "array",
                                            "items": {
                                                "type": "object",
                                                "properties": {
                                                    "id": { "type": "string" },
                                                    "name": { "type": "string" },
                                                    "skills": {
                                                        "type": "array",
                                                        "items": {
                                                            "type": "object",
                                                            "properties": {
                                                                "name": { "type": "string" },
                                                                "level": { "type": "number" },
                                                                "certifications": {
                                                                    "type": "array",
                                                                    "items": { "type": "string" }
                                                                }
                                                            },
                                                            "required": ["name", "level"]
                                                        }
                                                    }
                                                },
                                                "required": ["id", "name"]
                                            }
                                        },
                                        "projects": {
                                            "type": "array",
                                            "items": { "type": "string" }
                                        }
                                    },
                                    "required": ["teamName", "members"]
                                }
                            }
                        },
                        "required": ["name", "teams"]
                    }
                }
            },
            "required": ["id", "departments"]
        }
        "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("Organization".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type Organization = {
  id: string;
  departments: {
    name: string;
    teams: {
      teamName: string;
      members: {
        id: string;
        name: string;
        skills?: {
          name: string;
          level: number;
          certifications?: string[];
        }[];
      }[];
      projects?: string[];
    }[];
  }[];
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_nested_object_with_array_oneof() {
        let schema_json = r#"
          {
              "type": "object",
              "properties": {
                  "id": { "type": "string" },
                  "metadata": {
                      "type": "object",
                      "properties": {
                          "title": { "type": "string" },
                          "tags": {
                              "type": "array",
                              "items": {
                                  "oneOf": [
                                      { "type": "string" },
                                      {
                                          "type": "object",
                                          "properties": {
                                              "name": { "type": "string" },
                                              "value": { "type": "number" },
                                              "metadata": {
                                                  "type": "object",
                                                  "properties": {
                                                      "description": { "type": "string" },
                                                      "priority": { "type": "number" }
                                                  },
                                                  "required": ["description"]
                                              }
                                          },
                                          "required": ["name", "value"]
                                      }
                                  ]
                              }
                          }
                      },
                      "required": ["title", "tags"]
                  }
              },
              "required": ["id", "metadata"]
          }
          "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("DeepArray".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type DeepArray = {
  id: string;
  metadata: {
    title: string;
    tags: (string | {
      name: string;
      value: number;
      metadata?: {
        description: string;
        priority?: number;
      };
    })[];
  };
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_nested_object_with_array_allof() {
        let schema_json = r#"
          {
              "type": "object",
              "properties": {
                  "id": { "type": "string" },
                  "metadata": {
                      "type": "object",
                      "properties": {
                          "title": { "type": "string" },
                          "tags": {
                              "type": "array",
                              "items": {
                                  "allOf": [
                                      {
                                          "type": "object",
                                          "properties": {
                                              "id": { "type": "string" },
                                              "type": { "type": "string" }
                                          },
                                          "required": ["id"]
                                      },
                                      {
                                          "type": "object",
                                          "properties": {
                                              "metadata": {
                                                  "type": "object",
                                                  "properties": {
                                                      "description": { "type": "string" },
                                                      "created": { "type": "string" }
                                                  }
                                              }
                                          }
                                      }
                                  ]
                              }
                          }
                      },
                      "required": ["title", "tags"]
                  }
              },
              "required": ["id", "metadata"]
          }
          "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("DeepArrayAllOf".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type DeepArrayAllOf = {
  id: string;
  metadata: {
    title: string;
    tags: ({
      id: string;
      type?: string;
    } & {
      metadata?: {
        description?: string;
        created?: string;
      };
    })[];
  };
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_nested_object_with_array_anyof() {
        let schema_json = r#"
          {
              "type": "object",
              "properties": {
                  "id": { "type": "string" },
                  "metadata": {
                      "type": "object",
                      "properties": {
                          "title": { "type": "string" },
                          "tags": {
                              "type": "array",
                              "items": {
                                  "anyOf": [
                                      { "type": "string" },
                                      {
                                          "type": "object",
                                          "properties": {
                                              "name": { "type": "string" },
                                              "value": { "type": "number" },
                                              "metadata": {
                                                  "type": "object",
                                                  "properties": {
                                                      "description": { "type": "string" },
                                                      "priority": { "type": "number" }
                                                  },
                                                  "required": ["description"]
                                              }
                                          },
                                          "required": ["name", "value"]
                                      }
                                  ]
                              }
                          }
                      },
                      "required": ["title", "tags"]
                  }
              },
              "required": ["id", "metadata"]
          }
          "#;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("DeepArrayAny".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type DeepArrayAny = {
  id: string;
  metadata: {
    title: string;
    tags: (string | {
      name: string;
      value: number;
      metadata?: {
        description: string;
        priority?: number;
      };
    })[];
  };
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }

    #[test]
    fn test_object_with_deep_array_refs() {
        let schema_json = r##"
        {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "data": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "references": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/ExternalRef" }
                        }
                    },
                    "required": ["name", "references"]
                }
            },
            "required": ["id", "data"]
        }
        "##;

        let schema: Schema =
            serde_json::from_str(schema_json).expect("Could not deserialize schema");

        let type_interface =
            schema_to_typescript("DeepRefArray".to_string(), ReferenceOr::Item(schema));

        let expected = r##"type DeepRefArray = {
  id: string;
  data: {
    name: string;
    references: ExternalRef[];
  };
};"##;

        assert_eq!(type_interface.to_string(), expected.to_string());
    }
}
