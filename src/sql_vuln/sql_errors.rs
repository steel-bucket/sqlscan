// src/sql_vuln/sql_errors.rs
use regex::Regex;
use std::collections::HashMap;

pub struct SqlErrorChecker {
    patterns: HashMap<&'static str, Vec<Regex>>,
}

impl SqlErrorChecker {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // MySQL patterns
        patterns.insert(
            "MySQL",
            vec![
                Regex::new(r"SQL syntax.*MySQL").unwrap(),
                Regex::new(r"Warning.*mysql_.*").unwrap(),
                Regex::new(r"MySQL Query fail.*").unwrap(),
                Regex::new(r"SQL syntax.*MariaDB server").unwrap(),
            ],
        );

        // PostgreSQL patterns
        patterns.insert(
            "PostgreSQL",
            vec![
                Regex::new(r"PostgreSQL.*ERROR").unwrap(),
                Regex::new(r"Warning.*\Wpg_.*").unwrap(),
                Regex::new(r"Warning.*PostgreSQL").unwrap(),
            ],
        );

        // Microsoft SQL Server patterns
        patterns.insert(
            "Microsoft SQL Server",
            vec![
                Regex::new(r"OLE DB.* SQL Server").unwrap(),
                Regex::new(r"(\W|\A)SQL Server.*Driver").unwrap(),
                Regex::new(r"Warning.*odbc_.*").unwrap(),
                Regex::new(r"Warning.*mssql_").unwrap(),
                Regex::new(r"Msg \d+, Level \d+, State \d+").unwrap(),
                Regex::new(r"Unclosed quotation mark after the character string").unwrap(),
                Regex::new(r"Microsoft OLE DB Provider for ODBC Drivers").unwrap(),
            ],
        );

        // Microsoft Access patterns
        patterns.insert(
            "Microsoft Access",
            vec![
                Regex::new(r"Microsoft Access Driver").unwrap(),
                Regex::new(r"Access Database Engine").unwrap(),
                Regex::new(r"Microsoft JET Database Engine").unwrap(),
                Regex::new(r".*Syntax error.*query expression").unwrap(),
            ],
        );

        // Oracle patterns
        patterns.insert(
            "Oracle",
            vec![
                Regex::new(r"\bORA-[0-9][0-9][0-9][0-9]").unwrap(),
                Regex::new(r"Oracle error").unwrap(),
                Regex::new(r"Warning.*oci_.*").unwrap(),
                Regex::new(r"Microsoft OLE DB Provider for Oracle").unwrap(),
            ],
        );

        // IBM DB2 patterns
        patterns.insert(
            "IBM DB2",
            vec![
                Regex::new(r"CLI Driver.*DB2").unwrap(),
                Regex::new(r"DB2 SQL error").unwrap(),
            ],
        );

        // SQLite patterns
        patterns.insert(
            "SQLite",
            vec![
                Regex::new(r"SQLite/JDBCDriver").unwrap(),
                Regex::new(r"System.Data.SQLite.SQLiteException").unwrap(),
            ],
        );

        // Informix patterns
        patterns.insert(
            "Informix",
            vec![
                Regex::new(r"Warning.*ibase_.*").unwrap(),
                Regex::new(r"com.informix.jdbc").unwrap(),
            ],
        );

        // Sybase patterns
        patterns.insert(
            "Sybase",
            vec![
                Regex::new(r"Warning.*sybase.*").unwrap(),
                Regex::new(r"Sybase message").unwrap(),
            ],
        );

        Self { patterns }
    }

    pub fn check(&self, html: &str) -> (bool, Option<String>) {
        for (db_name, regexes) in &self.patterns {
            for regex in regexes {
                if regex.is_match(html) {
                    return (true, Some(db_name.to_string()));
                }
            }
        }
        (false, None)
    }
}

impl Default for SqlErrorChecker {
    fn default() -> Self {
        Self::new()
    }
}
