// benches/scanner_bench.rs
use crate::sql_vuln::{SqlErrorChecker, scanner::SqlInjectionScanner};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

fn bench_sql_error_detection(c: &mut Criterion) {
    let checker = SqlErrorChecker::new();
    let large_html = format!(
        r#"
    <html><head><title>Large Page</title></head><body>
    {}
    You have an error in your SQL syntax
    {}
    </body></html>
"#,
        "x".repeat(10000),
        "y".repeat(10000)
    );
    let test_cases = vec![
        (
            "mysql_error",
            r#"
            <html><body>
            You have an error in your SQL syntax; check the manual that corresponds to your MySQL server version
            </body></html>
        "#,
        ),
        (
            "postgresql_error",
            r#"
            <html><body>
            PostgreSQL query failed: ERROR:  syntax error at or near "'"
            </body></html>
        "#,
        ),
        (
            "mssql_error",
            r#"
            <html><body>
            Microsoft OLE DB Provider for ODBC Drivers error
            </body></html>
        "#,
        ),
        (
            "no_error",
            r#"
            <html><body>
            <p>Welcome to our website!</p>
            <p>This is a normal page with no SQL errors.</p>
            </body></html>
        "#,
        ),
        ("large_html", &large_html),
    ];

    let mut group = c.benchmark_group("sql_error_detection");

    for (name, html) in test_cases {
        group.bench_with_input(BenchmarkId::new("check", name), html, |b, html| {
            b.iter(|| checker.check(black_box(html)))
        });
    }

    group.finish();
}

fn bench_payload_generation(c: &mut Criterion) {
    let scanner = SqlInjectionScanner::new();

    c.bench_function("payload_generation", |b| {
        b.iter(|| {
            // Simulate payload generation and modification
            let base_url = "http://example.com/page.php?id=1&name=test";
            let payloads = vec!["'", "')", "';", "\"", "\")", "\";"];

            for payload in &payloads {
                let _modified = format!("{}{}", base_url, payload);
            }
        })
    });
}

fn bench_url_parsing(c: &mut Criterion) {
    let test_urls = vec![
        "http://example.com/page.php?id=1",
        "https://test.com/script.asp?user=admin&pass=secret",
        "http://vulnerable.com/index.jsp?category=books&sort=price",
        "https://shop.com/product.aspx?pid=123&color=red&size=large",
    ];

    let mut group = c.benchmark_group("url_parsing");

    for (i, url) in test_urls.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("parse", i), url, |b, url| {
            b.iter(|| {
                if let Ok(parsed) = url::Url::parse(black_box(url)) {
                    let _query = parsed.query().unwrap_or("");
                    let _host = parsed.host_str().unwrap_or("");
                    let _scheme = parsed.scheme();
                }
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_sql_error_detection,
    bench_payload_generation,
    bench_url_parsing
);
criterion_main!(benches);
