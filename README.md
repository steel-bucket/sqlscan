# SQLScan

**SQLScan** is a fast, modern, and extensible SQL injection vulnerability scanner written in Rust.

---

## Installation

### Prerequisites

- Rust (latest stable, [install here](https://rustup.rs/))
- Linux or macOS (Windows functionality coming soon)

### Build & Install

Clone the repository:

```sh
git clone https://github.com/steel-bucket/sqlscan.git
cd sqlscan
```

Build the project:

```sh
cargo build --release
```

Install system-wide (requires root):

```sh
sudo ./setup.sh
```

Or run directly:

```sh
cargo run -- [OPTIONS]
```

---

## Usage

### Basic Commands

- **Dork Search & Scan:**

  ```sh
  sqlscan -d "inurl:php?id=" -e google -p 50
  ```

- **Scan a Single Target:**

  ```sh
  sqlscan -t "http://example.com/page.php?id=1"
  ```

- **Reverse IP Scan:**

  ```sh
  sqlscan -t "example.com" -r
  ```

- **Save Results:**

  ```sh
  sqlscan -d "inurl:php?id=" -e bing -o results.json
  ```

### Options

| Option            | Description                                      |
|-------------------|--------------------------------------------------|
| `-t, --target`    | Scan target website                              |
| `-d, --dork`      | SQL injection dork (e.g., inurl:example)         |
| `-e, --engine`    | Search engine [google, bing, yahoo]              |
| `-p, --pages`     | Number of pages to search (default: 10)          |
| `-r, --reverse`   | Reverse domain lookup                            |
| `-o, --output`    | Output result to JSON file                       |
| `-s, --save-searches` | Save search results even if no vulnerabilities found |

### Examples

```sh
sqlscan -d "inurl:php?id=" -e google -p 20
sqlscan -t "example.com" -r
sqlscan -t "http://example.com/page.php?id=1"
```

---

## Output

- **Tabled Results**: Vulnerable URLs, database type, server, and language.
- **JSON Export**: Use `-o` to save results in JSON format.
- **Text Export**: Use `-s` to save all search results.

---

## Development

### Running Tests

```sh
cargo test
```

### Running Benchmarks(still WIP)

```sh
cargo bench
```



## Contribution Guidelines

We welcome contributions! Please follow these guidelines:

### 1. Pull Requests

- Fork the repository and create your branch from `master`.
- Write clear, concise commit messages.
- Add tests for new features or bug fixes.
- Run `cargo fmt` to format your code.
- Ensure all tests pass (`cargo test`).
- Submit your pull request with a description of your changes.

### 2. Code Style

- Follow Rust's standard formatting (`cargo fmt`).
- Use clear, descriptive variable and function names.
- Document public functions and modules.

### 3. Security

- Do not submit code that could be used for illegal or unethical purposes.
- Disclose vulnerabilities responsibly.

### 4. Communication

- Be respectful and constructive in discussions.
- Provide detailed information when reporting bugs or issues.

---
