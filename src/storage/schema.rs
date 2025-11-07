//! Arrow schema definitions for graph storage
//!
//! Defines the columnar layout for nodes and edges using Apache Arrow

use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;

/// Get the Arrow schema for node storage
///
/// Schema includes:
/// - id: UUID (stored as FixedSizeBinary(16))
/// - labels: List of strings
/// - properties: JSON-encoded map
pub fn node_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::FixedSizeBinary(16), false),
        Field::new(
            "labels",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, false))),
            false,
        ),
        Field::new("properties", DataType::Utf8, false), // JSON-encoded
        Field::new("created_at", DataType::Int64, false),
        Field::new("updated_at", DataType::Int64, false),
    ]))
}

/// Get the Arrow schema for edge storage
///
/// Schema includes:
/// - id: UUID (stored as FixedSizeBinary(16))
/// - from_id: UUID (source node)
/// - to_id: UUID (target node)
/// - relationship_type: String
/// - properties: JSON-encoded map
pub fn edge_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::FixedSizeBinary(16), false),
        Field::new("from_id", DataType::FixedSizeBinary(16), false),
        Field::new("to_id", DataType::FixedSizeBinary(16), false),
        Field::new("relationship_type", DataType::Utf8, false),
        Field::new("properties", DataType::Utf8, false), // JSON-encoded
        Field::new("created_at", DataType::Int64, false),
        Field::new("updated_at", DataType::Int64, false),
    ]))
}

/// Get the Arrow schema for label index
///
/// Maps labels to node IDs for efficient lookup
pub fn label_index_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("label", DataType::Utf8, false),
        Field::new("node_id", DataType::FixedSizeBinary(16), false),
    ]))
}

/// Get the Arrow schema for property index
///
/// Maps property keys/values to node IDs
pub fn property_index_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("key", DataType::Utf8, false),
        Field::new("value", DataType::Utf8, false), // JSON-encoded
        Field::new("node_id", DataType::FixedSizeBinary(16), false),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_schema() {
        let schema = node_schema();
        assert_eq!(schema.fields().len(), 5);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "labels");
        assert_eq!(schema.field(2).name(), "properties");
    }

    #[test]
    fn test_edge_schema() {
        let schema = edge_schema();
        assert_eq!(schema.fields().len(), 7);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "from_id");
        assert_eq!(schema.field(2).name(), "to_id");
        assert_eq!(schema.field(3).name(), "relationship_type");
    }
}

