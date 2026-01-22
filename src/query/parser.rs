//! Cypher query parser implementation
//!
//! Parses Cypher queries into AST using Pest

use crate::error::{DeepGraphError, Result};
use crate::graph::PropertyValue;
use crate::query::ast::*;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

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
        let pairs = CypherGrammarParser::parse(Rule::statement, query)
            .map_err(|e| DeepGraphError::ParserError(format!("Parse error: {}", e)))?;
        
        let pair = pairs.into_iter().next()
            .ok_or_else(|| DeepGraphError::ParserError("Empty parse result".to_string()))?;
        
        build_statement(pair)
    }
    
    /// Quick validation of query syntax
    pub fn validate(query: &str) -> Result<()> {
        CypherGrammarParser::parse(Rule::statement, query)
            .map_err(|e| DeepGraphError::ParserError(format!("Syntax error: {}", e)))?;
        Ok(())
    }
}

// ===================================================================
// AST Builder Functions
// ===================================================================

/// Build Statement from parse tree
fn build_statement(pair: Pair<Rule>) -> Result<Statement> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::query => return Ok(Statement::Query(build_query(inner)?)),
            _ => {}
        }
    }
    Err(DeepGraphError::ParserError("Invalid statement".to_string()))
}

/// Build Query from parse tree
fn build_query(pair: Pair<Rule>) -> Result<Query> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::read_query => return Ok(Query::Read(build_read_query(inner)?)),
            Rule::write_query => return Ok(Query::Write(build_write_query(inner)?)),
            _ => {}
        }
    }
    Err(DeepGraphError::ParserError("Invalid query".to_string()))
}

/// Build ReadQuery from parse tree (MATCH ... WHERE ... RETURN ...)
fn build_read_query(pair: Pair<Rule>) -> Result<ReadQuery> {
    let mut match_clause = None;
    let mut where_clause = None;
    let mut return_clause = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::match_clause => match_clause = Some(build_match_clause(inner)?),
            Rule::where_clause => where_clause = Some(build_where_clause(inner)?),
            Rule::return_clause => return_clause = Some(build_return_clause(inner)?),
            _ => {}
        }
    }
    
    Ok(ReadQuery {
        match_clause: match_clause
            .ok_or_else(|| DeepGraphError::ParserError("Missing MATCH clause".to_string()))?,
        where_clause,
        return_clause: return_clause
            .ok_or_else(|| DeepGraphError::ParserError("Missing RETURN clause".to_string()))?,
    })
}

/// Build MatchClause from parse tree
fn build_match_clause(pair: Pair<Rule>) -> Result<MatchClause> {
    let mut patterns = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::pattern {
            patterns.push(build_pattern(inner)?);
        }
    }
    
    Ok(MatchClause { patterns })
}

/// Build Pattern from parse tree
fn build_pattern(pair: Pair<Rule>) -> Result<Pattern> {
    let mut elements = Vec::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::node_pattern => {
                elements.push(PatternElement::Node(build_node_pattern(inner)?));
            }
            Rule::relationship_pattern => {
                elements.push(PatternElement::Relationship(build_relationship_pattern(inner)?));
            }
            _ => {}
        }
    }
    
    Ok(Pattern { elements })
}

/// Build NodePattern from parse tree: (n:Label {prop: value})
fn build_node_pattern(pair: Pair<Rule>) -> Result<NodePattern> {
    let mut variable = None;
    let mut labels = Vec::new();
    let mut properties = HashMap::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::variable => variable = Some(inner.as_str().to_string()),
            Rule::label_expression => {
                for label_pair in inner.into_inner() {
                    if label_pair.as_rule() == Rule::label {
                        labels.push(label_pair.as_str().to_string());
                    }
                }
            }
            Rule::properties => {
                for prop_pair in inner.into_inner() {
                    if prop_pair.as_rule() == Rule::property {
                        let (key, value) = build_property(prop_pair)?;
                        properties.insert(key, value);
                    }
                }
            }
            _ => {}
        }
    }
    
    Ok(NodePattern {
        variable,
        labels,
        properties,
    })
}

/// Build RelationshipPattern from parse tree
fn build_relationship_pattern(pair: Pair<Rule>) -> Result<RelationshipPattern> {
    let text = pair.as_str();
    let direction = if text.starts_with("<-") {
        Direction::Left
    } else if text.contains("->") {
        Direction::Right
    } else {
        Direction::Both
    };
    
    let mut variable = None;
    let mut rel_type = None;
    let mut properties = HashMap::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::variable => variable = Some(inner.as_str().to_string()),
            Rule::relationship_type => {
                for label_pair in inner.into_inner() {
                    if label_pair.as_rule() == Rule::label {
                        rel_type = Some(label_pair.as_str().to_string());
                    }
                }
            }
            Rule::properties => {
                for prop_pair in inner.into_inner() {
                    if prop_pair.as_rule() == Rule::property {
                        let (key, value) = build_property(prop_pair)?;
                        properties.insert(key, value);
                    }
                }
            }
            _ => {}
        }
    }
    
    Ok(RelationshipPattern {
        variable,
        rel_type,
        direction,
        properties,
    })
}

/// Build property (key: value) from parse tree
fn build_property(pair: Pair<Rule>) -> Result<(String, Expression)> {
    let mut key = String::new();
    let mut value = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::property_key => key = inner.as_str().to_string(),
            Rule::expression => value = Some(build_expression(inner)?),
            _ => {}
        }
    }
    
    value
        .map(|v| (key.clone(), v))
        .ok_or_else(|| DeepGraphError::ParserError(format!("Invalid property: {}", key)))
}

/// Build WhereClause from parse tree
fn build_where_clause(pair: Pair<Rule>) -> Result<WhereClause> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            return Ok(WhereClause {
                condition: build_expression(inner)?,
            });
        }
    }
    Err(DeepGraphError::ParserError("Invalid WHERE clause".to_string()))
}

/// Build ReturnClause from parse tree
fn build_return_clause(pair: Pair<Rule>) -> Result<ReturnClause> {
    let mut distinct = false;
    let mut items = Vec::new();
    let mut order_by = None;
    let mut limit = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::return_item => items.push(build_return_item(inner)?),
            Rule::order_clause => order_by = Some(build_order_clause(inner)?),
            Rule::limit_clause => limit = Some(build_limit_clause(inner)?),
            _ => {
                if inner.as_str().eq_ignore_ascii_case("DISTINCT") {
                    distinct = true;
                }
            }
        }
    }
    
    Ok(ReturnClause {
        distinct,
        items,
        order_by,
        limit,
    })
}

/// Build ReturnItem from parse tree
fn build_return_item(pair: Pair<Rule>) -> Result<ReturnItem> {
    let mut expression = None;
    let mut alias = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => expression = Some(build_expression(inner)?),
            Rule::identifier => alias = Some(inner.as_str().to_string()),
            _ => {}
        }
    }
    
    Ok(ReturnItem {
        expression: expression
            .ok_or_else(|| DeepGraphError::ParserError("Missing expression in RETURN".to_string()))?,
        alias,
    })
}

/// Build OrderClause from parse tree
fn build_order_clause(pair: Pair<Rule>) -> Result<Vec<OrderItem>> {
    let mut items = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::order_item {
            items.push(build_order_item(inner)?);
        }
    }
    
    Ok(items)
}

/// Build OrderItem from parse tree
fn build_order_item(pair: Pair<Rule>) -> Result<OrderItem> {
    let mut expression = None;
    let mut ascending = true;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => expression = Some(build_expression(inner)?),
            _ => {
                let text = inner.as_str();
                if text.eq_ignore_ascii_case("DESC") {
                    ascending = false;
                }
            }
        }
    }
    
    Ok(OrderItem {
        expression: expression
            .ok_or_else(|| DeepGraphError::ParserError("Missing expression in ORDER BY".to_string()))?,
        ascending,
    })
}

/// Build limit value from parse tree
fn build_limit_clause(pair: Pair<Rule>) -> Result<i64> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::integer {
            return inner.as_str().parse::<i64>()
                .map_err(|e| DeepGraphError::ParserError(format!("Invalid LIMIT value: {}", e)));
        }
    }
    Err(DeepGraphError::ParserError("Missing LIMIT value".to_string()))
}

/// Build Expression from parse tree (recursive)
fn build_expression(pair: Pair<Rule>) -> Result<Expression> {
    match pair.as_rule() {
        Rule::expression | Rule::or_expression | Rule::and_expression | 
        Rule::not_expression | Rule::comparison_expression | 
        Rule::additive_expression | Rule::multiplicative_expression |
        Rule::unary_expression => {
            // Recursively parse inner expressions
            let inner_pairs: Vec<_> = pair.into_inner().collect();
            
            if inner_pairs.len() == 1 {
                return build_expression(inner_pairs[0].clone());
            }
            
            if inner_pairs.len() == 3 {
                let left = build_expression(inner_pairs[0].clone())?;
                let op = inner_pairs[1].as_str();
                let right = build_expression(inner_pairs[2].clone())?;
                
                return Ok(match op.to_uppercase().as_str() {
                    "AND" => Expression::And(Box::new(left), Box::new(right)),
                    "OR" => Expression::Or(Box::new(left), Box::new(right)),
                    "=" => Expression::Eq(Box::new(left), Box::new(right)),
                    "!=" | "<>" => Expression::Ne(Box::new(left), Box::new(right)),
                    "<" => Expression::Lt(Box::new(left), Box::new(right)),
                    "<=" => Expression::Le(Box::new(left), Box::new(right)),
                    ">" => Expression::Gt(Box::new(left), Box::new(right)),
                    ">=" => Expression::Ge(Box::new(left), Box::new(right)),
                    "+" => Expression::Add(Box::new(left), Box::new(right)),
                    "-" => Expression::Sub(Box::new(left), Box::new(right)),
                    "*" => Expression::Mul(Box::new(left), Box::new(right)),
                    "/" => Expression::Div(Box::new(left), Box::new(right)),
                    "%" => Expression::Mod(Box::new(left), Box::new(right)),
                    _ => return Err(DeepGraphError::ParserError(format!("Unknown operator: {}", op))),
                });
            }
            
            // Single inner pair or NOT
            if inner_pairs.len() == 2 {
                let op = inner_pairs[0].as_str();
                if op.eq_ignore_ascii_case("NOT") {
                    let expr = build_expression(inner_pairs[1].clone())?;
                    return Ok(Expression::Not(Box::new(expr)));
                }
                // Unary minus
                if op == "-" {
                    let expr = build_expression(inner_pairs[1].clone())?;
                    return Ok(Expression::Neg(Box::new(expr)));
                }
            }
            
            // Fallback: try first inner
            if !inner_pairs.is_empty() {
                return build_expression(inner_pairs[0].clone());
            }
            
            Err(DeepGraphError::ParserError("Invalid expression".to_string()))
        }
        
        Rule::atom => {
            let inner = pair.into_inner().next()
                .ok_or_else(|| DeepGraphError::ParserError("Empty atom".to_string()))?;
            build_expression(inner)
        }
        
        Rule::literal => build_literal(pair),
        Rule::variable => Ok(Expression::Variable(pair.as_str().to_string())),
        Rule::property_lookup => build_property_lookup(pair),
        Rule::function_call => build_function_call(pair),
        Rule::parameter => Ok(Expression::Parameter(pair.as_str()[1..].to_string())),
        
        _ => Err(DeepGraphError::ParserError(format!("Unsupported expression: {:?}", pair.as_rule()))),
    }
}

/// Build literal value from parse tree
fn build_literal(pair: Pair<Rule>) -> Result<Expression> {
    let inner = pair.into_inner().next()
        .ok_or_else(|| DeepGraphError::ParserError("Empty literal".to_string()))?;
    
    let value = match inner.as_rule() {
        Rule::integer => {
            let i = inner.as_str().parse::<i64>()
                .map_err(|e| DeepGraphError::ParserError(format!("Invalid integer: {}", e)))?;
            PropertyValue::Integer(i)
        }
        Rule::float => {
            let f = inner.as_str().parse::<f64>()
                .map_err(|e| DeepGraphError::ParserError(format!("Invalid float: {}", e)))?;
            PropertyValue::Float(f)
        }
        Rule::string => {
            let s = inner.as_str();
            // Remove quotes
            let unquoted = &s[1..s.len()-1];
            PropertyValue::String(unquoted.to_string())
        }
        Rule::boolean => {
            let b = inner.as_str().eq_ignore_ascii_case("true");
            PropertyValue::Boolean(b)
        }
        Rule::null => PropertyValue::Null,
        _ => return Err(DeepGraphError::ParserError(format!("Unknown literal type: {:?}", inner.as_rule()))),
    };
    
    Ok(Expression::Literal(value))
}

/// Build property lookup (n.property) from parse tree
fn build_property_lookup(pair: Pair<Rule>) -> Result<Expression> {
    let mut variable = None;
    let mut property = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::variable => variable = Some(inner.as_str().to_string()),
            Rule::property_key => property = Some(inner.as_str().to_string()),
            _ => {}
        }
    }
    
    let var = variable.ok_or_else(|| DeepGraphError::ParserError("Missing variable in property lookup".to_string()))?;
    let prop = property.ok_or_else(|| DeepGraphError::ParserError("Missing property in property lookup".to_string()))?;
    
    Ok(Expression::Property(
        Box::new(Expression::Variable(var)),
        prop,
    ))
}

/// Build function call from parse tree
fn build_function_call(pair: Pair<Rule>) -> Result<Expression> {
    let mut name = String::new();
    let mut args = Vec::new();
    let mut distinct = false;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::expression => args.push(build_expression(inner)?),
            _ => {
                if inner.as_str().eq_ignore_ascii_case("DISTINCT") {
                    distinct = true;
                }
            }
        }
    }
    
    Ok(Expression::FunctionCall {
        name,
        args,
        distinct,
    })
}

/// Build WriteQuery from parse tree
fn build_write_query(pair: Pair<Rule>) -> Result<WriteQuery> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::create_clause => return Ok(WriteQuery::Create(build_create_clause(inner)?)),
            Rule::delete_clause => return Ok(WriteQuery::Delete(build_delete_clause(inner)?)),
            Rule::set_clause => return Ok(WriteQuery::Set(build_set_clause(inner)?)),
            Rule::merge_clause => return Ok(WriteQuery::Merge(build_merge_clause(inner)?)),
            _ => {}
        }
    }
    Err(DeepGraphError::ParserError("Invalid write query".to_string()))
}

/// Build CreateClause from parse tree
fn build_create_clause(pair: Pair<Rule>) -> Result<CreateClause> {
    let mut patterns = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::pattern {
            patterns.push(build_pattern(inner)?);
        }
    }
    
    Ok(CreateClause { patterns })
}

/// Build DeleteClause from parse tree
fn build_delete_clause(pair: Pair<Rule>) -> Result<DeleteClause> {
    let mut expressions = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            expressions.push(build_expression(inner)?);
        }
    }
    
    Ok(DeleteClause { expressions })
}

/// Build SetClause from parse tree
fn build_set_clause(pair: Pair<Rule>) -> Result<SetClause> {
    let mut items = Vec::new();
    
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::set_item {
            items.push(build_set_item(inner)?);
        }
    }
    
    Ok(SetClause { items })
}

/// Build SetItem from parse tree
fn build_set_item(pair: Pair<Rule>) -> Result<SetItem> {
    let mut variable = String::new();
    let mut property = String::new();
    let mut value = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::variable => variable = inner.as_str().to_string(),
            Rule::property_key => property = inner.as_str().to_string(),
            Rule::expression => value = Some(build_expression(inner)?),
            _ => {}
        }
    }
    
    Ok(SetItem {
        variable,
        property,
        value: value.ok_or_else(|| DeepGraphError::ParserError("Missing value in SET".to_string()))?,
    })
}

/// Build MergeClause from parse tree
fn build_merge_clause(pair: Pair<Rule>) -> Result<MergeClause> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::pattern {
            return Ok(MergeClause {
                pattern: build_pattern(inner)?,
            });
        }
    }
    Err(DeepGraphError::ParserError("Missing pattern in MERGE".to_string()))
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

    #[test]
    fn test_parse_simple_match() {
        let query = "MATCH (n) RETURN n;";
        let result = CypherParser::parse(query);
        assert!(result.is_ok());
        
        if let Ok(Statement::Query(Query::Read(read_query))) = result {
            assert!(read_query.match_clause.patterns.len() > 0);
            assert!(read_query.return_clause.items.len() > 0);
        } else {
            panic!("Expected ReadQuery");
        }
    }

    #[test]
    fn test_parse_match_with_label() {
        let query = "MATCH (n:Person) RETURN n;";
        let result = CypherParser::parse(query);
        assert!(result.is_ok());
        
        if let Ok(Statement::Query(Query::Read(read_query))) = result {
            let pattern = &read_query.match_clause.patterns[0];
            if let PatternElement::Node(node) = &pattern.elements[0] {
                assert_eq!(node.labels.len(), 1);
                assert_eq!(node.labels[0], "Person");
            } else {
                panic!("Expected Node pattern");
            }
        }
    }

    #[test]
    fn test_parse_match_with_where() {
        let query = "MATCH (n:Person) WHERE n.age > 25 RETURN n;";
        let result = CypherParser::parse(query);
        assert!(result.is_ok());
        
        if let Ok(Statement::Query(Query::Read(read_query))) = result {
            assert!(read_query.where_clause.is_some());
        }
    }

    #[test]
    fn test_parse_property_access() {
        let query = "MATCH (n:Person) RETURN n.name, n.age;";
        let result = CypherParser::parse(query);
        assert!(result.is_ok());
        
        if let Ok(Statement::Query(Query::Read(read_query))) = result {
            assert_eq!(read_query.return_clause.items.len(), 2);
        }
    }

    #[test]
    fn test_parse_with_limit() {
        let query = "MATCH (n:Person) RETURN n LIMIT 10;";
        let result = CypherParser::parse(query);
        assert!(result.is_ok());
        
        if let Ok(Statement::Query(Query::Read(read_query))) = result {
            assert_eq!(read_query.return_clause.limit, Some(10));
        }
    }

    #[test]
    fn test_parse_create() {
        let query = "CREATE (n:Person {name: \"Alice\", age: 30});";
        let result = CypherParser::parse(query);
        assert!(result.is_ok());
        
        if let Ok(Statement::Query(Query::Write(WriteQuery::Create(create)))) = result {
            assert!(create.patterns.len() > 0);
        }
    }
}

