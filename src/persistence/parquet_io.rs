//! Parquet I/O for graph data persistence
//!
//! Provides efficient serialization and deserialization of graph data
//! using Apache Parquet format.

use crate::error::{DeepGraphError, Result};
use arrow::array::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::Path;

/// Parquet writer for graph data
pub struct ParquetWriter {
    properties: WriterProperties,
}

impl ParquetWriter {
    /// Create a new Parquet writer with default properties
    pub fn new() -> Self {
        let properties = WriterProperties::builder()
            .set_compression(parquet::basic::Compression::SNAPPY)
            .build();
        
        Self { properties }
    }
    
    /// Create a new writer with custom properties
    pub fn with_properties(properties: WriterProperties) -> Self {
        Self { properties }
    }
    
    /// Write record batches to a Parquet file
    pub fn write_batches(
        &self,
        path: &Path,
        batches: &[RecordBatch],
    ) -> Result<()> {
        if batches.is_empty() {
            return Ok(());
        }
        
        let file = File::create(path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        let schema = batches[0].schema();
        let mut writer = ArrowWriter::try_new(
            file,
            schema,
            Some(self.properties.clone()),
        ).map_err(|e| DeepGraphError::StorageError(format!("Failed to create writer: {}", e)))?;
        
        for batch in batches {
            writer.write(batch)
                .map_err(|e| DeepGraphError::StorageError(format!("Failed to write batch: {}", e)))?;
        }
        
        writer.close()
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to close writer: {}", e)))?;
        
        Ok(())
    }
}

impl Default for ParquetWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Parquet reader for graph data
pub struct ParquetReader;

impl ParquetReader {
    /// Read record batches from a Parquet file
    pub fn read_batches(path: &Path) -> Result<Vec<RecordBatch>> {
        let file = File::open(path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to create reader: {}", e)))?;
        
        let reader = builder.build()
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to build reader: {}", e)))?;
        
        let mut batches = Vec::new();
        for batch_result in reader {
            let batch = batch_result
                .map_err(|e| DeepGraphError::StorageError(format!("Failed to read batch: {}", e)))?;
            batches.push(batch);
        }
        
        Ok(batches)
    }
    
    /// Get metadata from a Parquet file without reading the data
    pub fn read_metadata(path: &Path) -> Result<parquet::file::metadata::FileMetaData> {
        let file = File::open(path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to create reader: {}", e)))?;
        
        let metadata = builder.metadata();
        Ok(metadata.file_metadata().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int32Array, StringBuilder};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_write_and_read_parquet() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        // Create test data
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));
        
        let id_array = Int32Array::from(vec![1, 2, 3]);
        let mut name_builder = StringBuilder::new();
        name_builder.append_value("Alice");
        name_builder.append_value("Bob");
        name_builder.append_value("Charlie");
        let name_array = name_builder.finish();
        
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(id_array), Arc::new(name_array)],
        ).unwrap();
        
        // Write
        let writer = ParquetWriter::new();
        writer.write_batches(path, &[batch.clone()]).unwrap();
        
        // Read
        let batches = ParquetReader::read_batches(path).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].num_rows(), 3);
    }
}

