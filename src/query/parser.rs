use anyhow::Result;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

/// Parse a string to a collection of statements.
///
/// # Example
/// ```rust
/// use rookiedb::query::parser::parse;
/// let sql = "SELECT a, b, 123, myfunc(b) \
///            FROM table_1 \
///            WHERE a > b AND b < 100 \
///            ORDER BY a DESC, b";
/// let ast = parse(sql).unwrap();
/// println!("{:?}", ast);
/// ```
// todo(improve)： implement sql parser by self then romove sqlparser crate.
pub fn parse(sql: &str) -> Result<Vec<Statement>> {
    let dialect = GenericDialect {};
    let statement = Parser::parse_sql(&dialect, sql)?;
    Ok(statement)
}
