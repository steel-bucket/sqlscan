// benches/crawler_bench.rs
use crate::sql_vuln::crawler::WebCrawler;
use crate::web::useragents::UserAgents;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

fn bench_user_agent_selection(c: &mut Criterion) {
    let agents = UserAgents::new();

    c.bench_function("user_agent_selection", |b| b.iter(|| agents.get_random()));
}

fn bench_regex_matching(c: &mut Criterion) {
    use regex::Regex;

    let parameter_regex = Regex::new(r"(.*?)(.php\?|.asp\?|.aspx\?|.jsp\?)(.*?)=(.*)")
        .expect("Failed to compile regex");

    let test_urls = vec![
        "http://example.com/page.php?id=1",
        "https://test.com/script.asp?user=admin&pass=secret",
        "http://vulnerable.com/index.jsp?category=books",
        "https://shop.com/product.aspx?pid=123",
        "http://normal.com/static-page.html",
        "https://site.com/",
    ];

    let mut group = c.benchmark_group("regex_matching");

    for (i, url) in test_urls.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("match", i), url, |b, url| {
            b.iter(|| parameter_regex.is_match(black_box(url)))
        });
    }

    group.finish();
}

fn bench_url_resolution(c: &mut Criterion) {
    use url::Url;

    let base_url = "http://example.com/path/to/page.php";
    let relative_links = vec![
        "page1.php?id=1",
        "../admin.php?action=login",
        "/products.jsp?cat=tech",
        "http://other.com/external.asp?ref=example",
        "mailto:admin@example.com",
        "javascript:void(0)",
        "#section1",
        "?page=2&sort=date",
    ];

    let mut group = c.benchmark_group("url_resolution");

    for (i, link) in relative_links.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("resolve", i), link, |b, link| {
            b.iter(|| {
                if let Ok(base) = Url::parse(base_url) {
                    let _resolved = base.join(black_box(link));
                }
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_user_agent_selection,
    bench_regex_matching,
    bench_url_resolution
);
criterion_main!(benches);
