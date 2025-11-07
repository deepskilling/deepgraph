//! Abstract Syntax Tree for Cypher queries
//!
//! Represents the parsed structure of Cypher queries

use crate::graph::PropertyValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete Cypher statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Query(Query),
}

/// A query (read or write)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Query {
    Read(ReadQuery),
    Write(WriteQuery),
}

/// Read query (MATCH)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadQuery {
    pub match_clause: MatchClause,
    pub where_clause: Option<WhereClause>,
    pub return_clause: ReturnClause,
}

/// Write query (CREATE, DELETE, SET, MERGE)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WriteQuery {
    Create(CreateClause),
    Delete(DeleteClause),
    Set(SetClause),
    Merge(MergeClause),
}

/// MATCH clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchClause {
    pub patterns: Vec<Pattern>,
}

/// Pattern for graph matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub elements: Vec<PatternElement>,
}

/// Element in a pattern (node or relationship)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternElement {
    Node(NodePattern),
    Relationship(RelationshipPattern),
}

/// Node pattern in MATCH
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodePattern {
    pub variable: Option<String>,
    pub labels: Vec<String>,
    pub properties: HashMap<String, Expression>,
}

/// Relationship pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipPattern {
    pub variable: Option<String>,
    pub rel_type: Option<String>,
    pub direction: Direction,
    pub properties: HashMap<String, Expression>,
}

/// Direction of relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,      // <-
    Right,     // ->
    Both,      // -
}

/// WHERE clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhereClause {
    pub condition: Expression,
}

/// RETURN clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnClause {
    pub distinct: bool,
    pub items: Vec<ReturnItem>,
    pub order_by: Option<Vec<OrderItem>>,
    pub limit: Option<i64>,
}

/// Item in RETURN clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnItem {
    pub expression: Expression,
    pub alias: Option<String>,
}

/// ORDER BY item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderItem {
    pub expression: Expression,
    pub ascending: bool,
}

/// CREATE clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateClause {
    pub patterns: Vec<Pattern>,
}

/// DELETE clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteClause {
    pub expressions: Vec<Expression>,
}

/// SET clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetClause {
    pub items: Vec<SetItem>,
}

/// Item in SET clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetItem {
    pub variable: String,
    pub property: String,
    pub value: Expression,
}

/// MERGE clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MergeClause {
    pub pattern: Pattern,
}

/// Expression (literals, variables, operators, functions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    // Literals
    Literal(PropertyValue),
    
    // Variables
    Variable(String),
    
    // Property access
    Property(Box<Expression>, String),
    
    // Binary operations
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    
    // Comparison
    Eq(Box<Expression>, Box<Expression>),
    Ne(Box<Expression>, Box<Expression>),
    Lt(Box<Expression>, Box<Expression>),
    Le(Box<Expression>, Box<Expression>),
    Gt(Box<Expression>, Box<Expression>),
    Ge(Box<Expression>, Box<Expression>),
    
    // Arithmetic
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    
    // Unary
    Not(Box<Expression>),
    Neg(Box<Expression>),
    
    // Function call
    FunctionCall {
        name: String,
        args: Vec<Expression>,
        distinct: bool,
    },
    
    // Parameter
    Parameter(String),
}

impl Expression {
    /// Create a literal expression
    pub fn literal(value: PropertyValue) -> Self {
        Expression::Literal(value)
    }
    
    /// Create a variable expression
    pub fn variable(name: impl Into<String>) -> Self {
        Expression::Variable(name.into())
    }
    
    /// Create a property access expression
    pub fn property(expr: Expression, prop: impl Into<String>) -> Self {
        Expression::Property(Box::new(expr), prop.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_literal() {
        let expr = Expression::literal(PropertyValue::Integer(42));
        match expr {
            Expression::Literal(PropertyValue::Integer(42)) => (),
            _ => panic!("Expected literal integer"),
        }
    }

    #[test]
    fn test_expression_variable() {
        let expr = Expression::variable("n");
        match expr {
            Expression::Variable(ref name) if name == "n" => (),
            _ => panic!("Expected variable n"),
        }
    }

    #[test]
    fn test_expression_property() {
        let expr = Expression::property(Expression::variable("n"), "name");
        match expr {
            Expression::Property(_, ref prop) if prop == "name" => (),
            _ => panic!("Expected property access"),
        }
    }

    #[test]
    fn test_node_pattern() {
        let pattern = NodePattern {
            variable: Some("n".to_string()),
            labels: vec!["Person".to_string()],
            properties: HashMap::new(),
        };
        
        assert_eq!(pattern.variable, Some("n".to_string()));
        assert_eq!(pattern.labels.len(), 1);
    }

    #[test]
    fn test_return_clause() {
        let return_clause = ReturnClause {
            distinct: false,
            items: vec![ReturnItem {
                expression: Expression::variable("n"),
                alias: None,
            }],
            order_by: None,
            limit: None,
        };
        
        assert!(!return_clause.distinct);
        assert_eq!(return_clause.items.len(), 1);
    }
}

