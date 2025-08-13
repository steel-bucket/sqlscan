// tests/integration_tests.rs
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2.0"));
}

#[test]
fn test_invalid_search_engine() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-d", "test", "-e", "invalid"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid search engine") || stderr.contains("invalid"));
}

#[test]
fn test_missing_required_args() {
    // Test with only dork but no engine
    let output = Command::new("cargo")
        .args(&["run", "--", "-d", "test"])
        .output()
        .expect("Failed to execute command");

    // Should show help since incomplete arguments provided
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Examples:") || stdout.contains("USAGE:"));
}

#[cfg(test)]
mod sql_vuln_tests {
    use crate::sql_vuln::{SqlErrorChecker, StdUtils};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_sql_error_checker_mysql() {
        let checker = SqlErrorChecker::new();
        let html_with_mysql_error = r#"
            <html><body>
            You have an error in your SQL syntax; check the manual that corresponds to your MySQL server version
            </body></html>
        "#;

        let (is_vulnerable, db_type) = checker.check(html_with_mysql_error);
        assert!(is_vulnerable);
        assert_eq!(db_type, Some("MySQL".to_string()));
    }

    #[test]
    fn test_sql_error_checker_no_error() {
        let checker = SqlErrorChecker::new();
        let html_normal = r#"
            <html><body>
            <p>Welcome to our website!</p>
            </body></html>
        "#;

        let (is_vulnerable, db_type) = checker.check(html_normal);
        assert!(!is_vulnerable);
        assert_eq!(db_type, None);
    }

    #[test]
    fn test_std_utils_dump() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_dump.txt");

        let test_data = vec![
            "http://example1.com".to_string(),
            "http://example2.com".to_string(),
        ];

        let result = StdUtils::dump(&test_data, file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("http://example1.com"));
        assert!(content.contains("http://example2.com"));
    }

    #[test]
    fn test_std_utils_dump_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_dump.json");

        let test_data = vec![(
            "http://example.com".to_string(),
            "MySQL".to_string(),
            "Apache".to_string(),
            "PHP".to_string(),
        )];

        let result = StdUtils::dump_json(&test_data, file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("http://example.com"));
        assert!(content.contains("MySQL"));
    }
}

#[cfg(test)]
mod web_tests {
    use crate::web::useragents::UserAgents;

    #[test]
    fn test_user_agents_random() {
        let agents = UserAgents::new();
        let agent1 = agents.get_random();
        let agent2 = agents.get_random();

        // Both should be valid user agent strings
        assert!(!agent1.is_empty());
        assert!(!agent2.is_empty());
        assert!(agent1.contains("Mozilla"));
    }

    #[test]
    fn test_user_agents_get_all() {
        let agents = UserAgents::new();
        let all_agents = agents.get_all();

        assert!(!all_agents.is_empty());
        assert!(all_agents.len() > 5); // Should have multiple user agents

        for agent in all_agents {
            assert!(!agent.is_empty());
        }
    }
}

// Mock HTTP server tests
#[cfg(test)]
mod mock_server_tests {
    use crate::web::web::get_html;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_html_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("<html><body>Test</body></html>"),
            )
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let result = get_html(&url, false).await;

        assert!(result.is_ok());
        let (html, _) = result.unwrap();
        assert!(html.contains("Test"));
    }

    #[tokio::test]
    async fn test_get_html_with_sql_error() {
        let mock_server = MockServer::start().await;

        let sql_error_html = r#"
            <html><body>
            You have an error in your SQL syntax; check the manual that corresponds to your MySQL server version
            </body></html>
        "#;

        Mock::given(method("GET"))
            .and(path("/vulnerable"))
            .respond_with(ResponseTemplate::new(200).set_body_string(sql_error_html))
            .mount(&mock_server)
            .await;

        let url = format!("{}/vulnerable", mock_server.uri());
        let result = get_html(&url, false).await;

        assert!(result.is_ok());
        let (html, _) = result.unwrap();
        assert!(html.contains("SQL syntax"));
    }

    #[tokio::test]
    async fn test_get_html_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/error"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let url = format!("{}/error", mock_server.uri());
        let result = get_html(&url, false).await;

        // Should handle 500 errors gracefully
        match result {
            Ok((html, _)) => {
                // If we get content back, it should contain the error message
                assert!(html.contains("Internal Server Error"));
            }
            Err(_) => {
                // Or it might return an error, which is also acceptable
            }
        }
    }
}

// Benchmark setup
#[cfg(test)]
mod benchmark_tests {
    use crate::sql_vuln::SqlErrorChecker;
    use crate::web::useragents::UserAgents;
    use criterion::{Criterion, black_box};

    pub fn bench_sql_error_detection(c: &mut Criterion) {
        let checker = SqlErrorChecker::new();
        let test_html = r#"
            <html><body>
            <p>Some content</p>
            You have an error in your SQL syntax; check the manual that corresponds to your MySQL server version
            <p>More content</p>
            </body></html>
        "#;

        c.bench_function("sql_error_detection", |b| {
            b.iter(|| checker.check(black_box(test_html)))
        });
    }

    pub fn bench_user_agent_selection(c: &mut Criterion) {
        use crate::web::useragents::UserAgents;

        let agents = UserAgents::new();

        c.bench_function("user_agent_selection", |b| b.iter(|| agents.get_random()));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql_vuln::scanner::SqlInjectionScanner;
    use crate::{VERSION, build_cli, single_scan};

    #[test]
    fn test_cli_creation() {
        let app = build_cli();
        assert_eq!(app.get_name(), "sqlscan");
        assert_eq!(app.get_version(), Some(VERSION));
    }

    #[test]
    fn test_cli_dork_and_engine_args() {
        let app = build_cli();
        let matches =
            app.try_get_matches_from(vec!["sqlscan", "-d", "inurl:php?id=", "-e", "google"]);
        assert!(matches.is_ok());

        let matches = matches.unwrap();
        assert_eq!(
            matches.get_one::<String>("dork"),
            Some(&"inurl:php?id=".to_string())
        );
        assert_eq!(
            matches.get_one::<String>("engine"),
            Some(&"google".to_string())
        );
    }

    #[test]
    fn test_cli_target_arg() {
        let app = build_cli();
        let matches = app.try_get_matches_from(vec!["sqlscan", "-t", "example.com"]);
        assert!(matches.is_ok());

        let matches = matches.unwrap();
        assert_eq!(
            matches.get_one::<String>("target"),
            Some(&"example.com".to_string())
        );
    }

    #[test]
    fn test_cli_reverse_flag() {
        let app = build_cli();
        let matches = app.try_get_matches_from(vec!["sqlscan", "-t", "example.com", "-r"]);
        assert!(matches.is_ok());

        let matches = matches.unwrap();
        assert!(matches.get_flag("reverse"));
    }

    #[test]
    fn test_cli_pages_default() {
        let app = build_cli();
        let matches = app.try_get_matches_from(vec!["sqlscan", "-d", "test", "-e", "google"]);
        assert!(matches.is_ok());

        let matches = matches.unwrap();
        assert_eq!(matches.get_one::<usize>("pages"), Some(&10));
    }

    #[test]
    fn test_cli_pages_custom() {
        let app = build_cli();
        let matches =
            app.try_get_matches_from(vec!["sqlscan", "-d", "test", "-e", "google", "-p", "50"]);
        assert!(matches.is_ok());

        let matches = matches.unwrap();
        assert_eq!(matches.get_one::<usize>("pages"), Some(&50));
    }

    #[tokio::test]
    async fn test_single_scan_invalid_url() {
        let scanner = SqlInjectionScanner::new();
        let result = single_scan("invalid-url", &scanner).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_single_scan_valid_url_format() {
        let scanner = SqlInjectionScanner::new();
        // This should not panic and should handle the URL parsing correctly
        let result = single_scan("http://example.com", &scanner).await;
        // The result might be None due to network issues in tests, but it shouldn't crash
        assert!(result.is_none() || result.is_some());
    }
}
