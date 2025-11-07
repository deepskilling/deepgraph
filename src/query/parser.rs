//! Cypher query parser implementation
//!
//! Parses Cypher queries into AST using Pest

use crate::error::{DeepGraphError, Result};
use crate::query::ast::*;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "query/grammar.pest"]
pub struct CypherGrammarParser;

/// Main Cypher parser
pub struct CypherParser;

impl CypherParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self
    }
    
    /// Parse a Cypher query string into an AST
    pub fn parse(query: &str) -> Result<Statement> {
        let _pairs = CypherGrammarParser::parse(Rule::statement, query)
            .map_err(|e| DeepGraphError::ParserError(format!("Parse error: {}", e)))?;
        
        // For now, return a placeholder until we implement full parsing
        // This allows the code to compile while we build out the parser
        Ok(Statement::Query(Query::Read(ReadQuery {
            match_clause: MatchClause {
                patterns: vec![],
            },
            where_clause: None,
            return_clause: ReturnClause {
                distinct: false,
                items: vec![],
                order_by: None,
                limit: None,
            },
        })))
    }
    
    /// Quick validation of query syntax
    pub fn validate(query: &str) -> Result<()> {
        CypherGrammarParser::parse(Rule::statement, query)
            .map_err(|e| DeepGraphError::ParserError(format!("Syntax error: {}", e)))?;
        Ok(())
    }
}

impl Default for CypherParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let _parser = CypherParser::new();
        // Parser created successfully
    }

    #[test]
    fn test_simple_match_validation() {
        let query = "MATCH (n) RETURN n;";
        assert!(CypherParser::validate(query).is_ok());
    }

    #[test]
    fn test_invalid_query() {
        let query = "INVALID SYNTAX!!!";
        assert!(CypherParser::validate(query).is_err());
    }

    #[test]
    fn test_match_with_label() {
        let query = "MATCH (n:Person) RETURN n;";
        assert!(CypherParser::validate(query).is_ok());
    }

    #[test]
    fn test_match_with_where() {
        let query = "MATCH (n:Person) WHERE n.age > 25 RETURN n;";
        assert!(CypherParser::validate(query).is_ok());
    }

    #[test]
    fn test_create_query() {
        let query = "CREATE (n:Person {name: \"Alice\"});";
        assert!(CypherParser::validate(query).is_ok());
    }
}

