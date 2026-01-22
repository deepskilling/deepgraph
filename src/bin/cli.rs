//! DeepGraph Interactive CLI and REPL

use clap::Parser;
use deepgraph::{
    storage::{DiskStorage, MemoryStorage, StorageBackend},
    query::{CypherParser, QueryPlanner, QueryExecutor},
    import::{CsvImporter, JsonImporter},
};
use prettytable::{Table, Row, Cell, format};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::sync::Arc;
use std::time::Instant;

/// DeepGraph Interactive CLI
#[derive(Parser)]
#[command(name = "deepgraph-cli")]
#[command(about = "DeepGraph - Interactive Graph Database CLI", long_about = None)]
#[command(version)]
struct Cli {
    /// Database path (default: in-memory)
    #[arg(short, long)]
    database: Option<String>,
    
    /// Execute query and exit
    #[arg(short, long)]
    query: Option<String>,
    
    /// Read queries from file
    #[arg(short, long)]
    file: Option<String>,
    
    /// Output format: table, json, csv
    #[arg(long, default_value = "table")]
    output: String,
    
    /// Import CSV nodes file
    #[arg(long)]
    import_csv_nodes: Option<String>,
    
    /// Import CSV edges file  
    #[arg(long)]
    import_csv_edges: Option<String>,
    
    /// Import JSON nodes file
    #[arg(long)]
    import_json_nodes: Option<String>,
    
    /// Import JSON edges file
    #[arg(long)]
    import_json_edges: Option<String>,
}

fn main() {
    env_logger::init();
    
    let cli = Cli::parse();
    
    // Handle imports first
    if cli.import_csv_nodes.is_some() || cli.import_json_nodes.is_some() {
        handle_imports(&cli);
        return;
    }
    
    // Handle single query
    if let Some(ref query) = cli.query {
        handle_single_query(&cli, query);
        return;
    }
    
    // Handle file queries
    if let Some(ref file_path) = cli.file {
        handle_file_queries(&cli, file_path);
        return;
    }
    
    // Start interactive REPL
    start_repl(&cli);
}

fn handle_imports(cli: &Cli) {
    println!("DeepGraph Data Import");
    println!("====================\n");
    
    let db_path = cli.database.as_deref().unwrap_or("./deepgraph_import.db");
    
    match DiskStorage::new(db_path) {
        Ok(storage) => {
            let mut node_id_map = std::collections::HashMap::new();
            
            // Import CSV nodes
            if let Some(nodes_file) = &cli.import_csv_nodes {
                println!("Importing CSV nodes from: {}", nodes_file);
                let importer = CsvImporter::new();
                match importer.import_nodes(&storage, nodes_file) {
                    Ok(stats) => {
                        println!("✅ Imported {} nodes in {}ms", stats.nodes_imported, stats.duration_ms);
                        if !stats.errors.is_empty() {
                            println!("⚠️  {} errors encountered", stats.errors.len());
                        }
                        node_id_map = stats.node_id_map;
                    }
                    Err(e) => {
                        eprintln!("❌ Error importing nodes: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            
            // Import CSV edges
            if let Some(edges_file) = &cli.import_csv_edges {
                println!("Importing CSV edges from: {}", edges_file);
                let importer = CsvImporter::new();
                match importer.import_edges(&storage, edges_file, &node_id_map) {
                    Ok(stats) => {
                        println!("✅ Imported {} edges in {}ms", stats.edges_imported, stats.duration_ms);
                        if !stats.errors.is_empty() {
                            println!("⚠️  {} errors encountered", stats.errors.len());
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Error importing edges: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            
            // Import JSON nodes
            if let Some(nodes_file) = &cli.import_json_nodes {
                println!("Importing JSON nodes from: {}", nodes_file);
                let importer = JsonImporter::new();
                match importer.import_nodes(&storage, nodes_file) {
                    Ok(stats) => {
                        println!("✅ Imported {} nodes in {}ms", stats.nodes_imported, stats.duration_ms);
                        if !stats.errors.is_empty() {
                            println!("⚠️  {} errors encountered", stats.errors.len());
                        }
                        node_id_map = stats.node_id_map;
                    }
                    Err(e) => {
                        eprintln!("❌ Error importing nodes: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            
            // Import JSON edges
            if let Some(edges_file) = &cli.import_json_edges {
                println!("Importing JSON edges from: {}", edges_file);
                let importer = JsonImporter::new();
                match importer.import_edges(&storage, edges_file, &node_id_map) {
                    Ok(stats) => {
                        println!("✅ Imported {} edges in {}ms", stats.edges_imported, stats.duration_ms);
                        if !stats.errors.is_empty() {
                            println!("⚠️  {} errors encountered", stats.errors.len());
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Error importing edges: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            
            println!("\n✅ Import complete! Database: {}", db_path);
            println!("Nodes: {}", storage.node_count());
            println!("Edges: {}", storage.edge_count());
        }
        Err(e) => {
            eprintln!("❌ Failed to open database: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_single_query(cli: &Cli, query: &str) {
    let db_path = cli.database.as_deref();
    
    if let Some(path) = db_path {
        // Use disk storage
        match DiskStorage::new(path) {
            Ok(storage) => {
                let storage = Arc::new(storage);
                match execute_cypher_query_disk(query, &storage) {
                    Ok(result) => {
                        match cli.output.as_str() {
                            "json" => print_json_output(&result),
                            "csv" => print_csv_output(&result),
                            _ => print_table_output(&result),
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Query error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to open database: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Use memory storage
        let storage = Arc::new(MemoryStorage::new());
        match execute_cypher_query_memory(query, &storage) {
            Ok(result) => {
                match cli.output.as_str() {
                    "json" => print_json_output(&result),
                    "csv" => print_csv_output(&result),
                    _ => print_table_output(&result),
                }
            }
            Err(e) => {
                eprintln!("❌ Query error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn handle_file_queries(_cli: &Cli, _file_path: &str) {
    println!("File query execution not yet implemented");
    println!("Use -q for single queries or start interactive REPL without arguments");
}

fn start_repl(cli: &Cli) {
    println!("DeepGraph REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type :help for help, :exit to quit\n");
    
    let db_path = cli.database.as_deref();
    
    if let Some(path) = db_path {
        // Disk storage REPL
        match DiskStorage::new(path) {
            Ok(storage) => {
                println!("✅ Opened database: {}", path);
                start_repl_disk(Arc::new(storage));
            }
            Err(e) => {
                eprintln!("❌ Failed to open database: {}", e);
                eprintln!("Starting with in-memory storage instead...\n");
                start_repl_memory(Arc::new(MemoryStorage::new()));
            }
        }
    } else {
        // Memory storage REPL
        println!("Using in-memory storage (data will not persist)");
        println!("Use --database <path> for persistent storage\n");
        start_repl_memory(Arc::new(MemoryStorage::new()));
    }
}

fn start_repl_memory(storage: Arc<MemoryStorage>) {
    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");
    
    loop {
        let readline = rl.readline("deepgraph> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                
                if trimmed.is_empty() {
                    continue;
                }
                
                let _ = rl.add_history_entry(&line);
                
                if trimmed.starts_with(':') {
                    handle_meta_command_memory(trimmed, &storage);
                    continue;
                }
                
                match execute_cypher_query_memory(trimmed, &storage) {
                    Ok(result) => {
                        print_table_output(&result);
                    }
                    Err(e) => {
                        eprintln!("❌ Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn start_repl_disk(storage: Arc<DiskStorage>) {
    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");
    
    loop {
        let readline = rl.readline("deepgraph> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                
                if trimmed.is_empty() {
                    continue;
                }
                
                let _ = rl.add_history_entry(&line);
                
                if trimmed.starts_with(':') {
                    handle_meta_command_disk(trimmed, &storage);
                    continue;
                }
                
                match execute_cypher_query_disk(trimmed, &storage) {
                    Ok(result) => {
                        print_table_output(&result);
                    }
                    Err(e) => {
                        eprintln!("❌ Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn handle_meta_command_memory(cmd: &str, storage: &Arc<MemoryStorage>) {
    handle_meta_command_impl(cmd, storage.node_count(), storage.edge_count());
}

fn handle_meta_command_disk(cmd: &str, storage: &Arc<DiskStorage>) {
    handle_meta_command_impl(cmd, storage.node_count(), storage.edge_count());
}

fn handle_meta_command_impl(cmd: &str, node_count: usize, edge_count: usize) {
    match cmd.trim() {
        ":help" => {
            println!("\nDeepGraph REPL Commands:");
            println!("  Cypher Queries:");
            println!("    MATCH (n) RETURN n        - Execute Cypher query");
            println!("");
            println!("  Meta Commands:");
            println!("    :help                     - Show this help");
            println!("    :exit, :quit              - Exit REPL");
            println!("    :stats                    - Show database statistics");
            println!("    :clear                    - Clear screen");
            println!("");
            println!("  Examples:");
            println!("    MATCH (n:Person) RETURN n.name, n.age;");
            println!("    MATCH (n) WHERE n.age > 25 RETURN n;");
            println!("");
        }
        ":stats" => {
            println!("\nDatabase Statistics:");
            println!("  Nodes: {}", node_count);
            println!("  Edges: {}", edge_count);
            println!("");
        }
        ":exit" | ":quit" => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        ":clear" => {
            print!("\x1B[2J\x1B[1;1H");  // ANSI escape codes to clear screen
        }
        _ => {
            println!("Unknown command: {}", cmd);
            println!("Type :help for available commands");
        }
    }
}

fn execute_cypher_query_memory(query: &str, storage: &Arc<MemoryStorage>) -> Result<QueryResult, String> {
    let start = Instant::now();
    
    let ast = CypherParser::parse(query)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let query_ast = match ast {
        deepgraph::query::ast::Statement::Query(q) => q,
    };
    
    let planner = QueryPlanner::new();
    let logical_plan = planner.logical_plan(&query_ast)
        .map_err(|e| format!("Planning error: {}", e))?;
    let physical_plan = planner.physical_plan(&logical_plan)
        .map_err(|e| format!("Physical planning error: {}", e))?;
    
    let executor = QueryExecutor::new(storage.clone());
    let result = executor.execute(&physical_plan)
        .map_err(|e| format!("Execution error: {}", e))?;
    
    let duration = start.elapsed();
    
    Ok(QueryResult {
        columns: result.columns,
        rows: result.rows,
        row_count: result.row_count,
        duration_ms: duration.as_millis() as u64,
    })
}

fn execute_cypher_query_disk(query: &str, storage: &Arc<DiskStorage>) -> Result<QueryResult, String> {
    let start = Instant::now();
    
    let ast = CypherParser::parse(query)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let query_ast = match ast {
        deepgraph::query::ast::Statement::Query(q) => q,
    };
    
    let planner = QueryPlanner::new();
    let logical_plan = planner.logical_plan(&query_ast)
        .map_err(|e| format!("Planning error: {}", e))?;
    let physical_plan = planner.physical_plan(&logical_plan)
        .map_err(|e| format!("Physical planning error: {}", e))?;
    
    let executor = QueryExecutor::new(storage.clone());
    let result = executor.execute(&physical_plan)
        .map_err(|e| format!("Execution error: {}", e))?;
    
    let duration = start.elapsed();
    
    Ok(QueryResult {
        columns: result.columns,
        rows: result.rows,
        row_count: result.row_count,
        duration_ms: duration.as_millis() as u64,
    })
}

struct QueryResult {
    columns: Vec<String>,
    rows: Vec<std::collections::HashMap<String, deepgraph::graph::PropertyValue>>,
    row_count: usize,
    duration_ms: u64,
}

fn print_table_output(result: &QueryResult) {
    if result.rows.is_empty() {
        println!("(no results)");
        return;
    }
    
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    
    // Add header
    table.add_row(Row::new(
        result.columns.iter()
            .map(|c| Cell::new(c))
            .collect()
    ));
    
    // Add rows
    for row in &result.rows {
        table.add_row(Row::new(
            result.columns.iter()
                .map(|col| {
                    let value = row.get(col)
                        .map(|v| format_property_value(v))
                        .unwrap_or_else(|| "NULL".to_string());
                    Cell::new(&value)
                })
                .collect()
        ));
    }
    
    println!("{}", table);
    println!("{} row(s) ({}ms)", result.row_count, result.duration_ms);
}

fn print_json_output(result: &QueryResult) {
    let json_rows: Vec<serde_json::Value> = result.rows.iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (key, value) in row {
                obj.insert(key.clone(), property_value_to_json(value));
            }
            serde_json::Value::Object(obj)
        })
        .collect();
    
    println!("{}", serde_json::to_string_pretty(&json_rows).unwrap());
}

fn print_csv_output(result: &QueryResult) {
    // Print header
    println!("{}", result.columns.join(","));
    
    // Print rows
    for row in &result.rows {
        let values: Vec<String> = result.columns.iter()
            .map(|col| {
                row.get(col)
                    .map(|v| format_property_value(v))
                    .unwrap_or_else(|| "".to_string())
            })
            .collect();
        println!("{}", values.join(","));
    }
}

fn format_property_value(value: &deepgraph::graph::PropertyValue) -> String {
    use deepgraph::graph::PropertyValue;
    match value {
        PropertyValue::String(s) => s.clone(),
        PropertyValue::Integer(i) => i.to_string(),
        PropertyValue::Float(f) => format!("{:.2}", f),
        PropertyValue::Boolean(b) => b.to_string(),
        PropertyValue::Null => "NULL".to_string(),
        PropertyValue::List(list) => format!("{:?}", list),
        PropertyValue::Map(map) => format!("{:?}", map),
    }
}

fn property_value_to_json(value: &deepgraph::graph::PropertyValue) -> serde_json::Value {
    use deepgraph::graph::PropertyValue;
    match value {
        PropertyValue::String(s) => serde_json::Value::String(s.clone()),
        PropertyValue::Integer(i) => serde_json::Value::Number((*i).into()),
        PropertyValue::Float(f) => {
            serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into()))
        }
        PropertyValue::Boolean(b) => serde_json::Value::Bool(*b),
        PropertyValue::Null => serde_json::Value::Null,
        PropertyValue::List(list) => {
            serde_json::Value::Array(list.iter().map(property_value_to_json).collect())
        }
        PropertyValue::Map(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), property_value_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
    }
}
