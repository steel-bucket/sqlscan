// src/sql_vuln/std_utils.rs
use chrono::Utc;
use colored::{Color, Colorize};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use tabled::{Table, Tabled};

#[derive(Debug, Clone, Tabled)]
pub struct VulnData {
    #[tabled(rename = "Index")]
    pub index: usize,
    #[tabled(rename = "URL")]
    pub url: String,
    #[tabled(rename = "Database")]
    pub database: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct ServerData {
    #[tabled(rename = "Website")]
    pub website: String,
    #[tabled(rename = "Server")]
    pub server: String,
    #[tabled(rename = "Language")]
    pub language: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct FullVulnData {
    #[tabled(rename = "Index")]
    pub index: usize,
    #[tabled(rename = "URL")]
    pub url: String,
    #[tabled(rename = "Database")]
    pub database: String,
    #[tabled(rename = "Server")]
    pub server: String,
    #[tabled(rename = "Language")]
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonVulnData {
    pub url: String,
    pub db: String,
    pub server: String,
    pub lang: String,
}

pub type TableData = Vec<Vec<String>>;

pub struct StdUtils;

impl StdUtils {
    pub fn stdin(message: &str, params: &[&str], upper: bool, lower: bool) -> String {
        loop {
            let symbol = "[OPT]".color(Color::Magenta);
            let time = format!("[{}]", Utc::now().format("%H:%M:%S")).color(Color::Green);

            print!("{} {} {}: ", symbol, time, message);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let mut option = input.trim().to_string();

            if upper {
                option = option.to_uppercase();
            } else if lower {
                option = option.to_lowercase();
            }

            if params.contains(&option.as_str()) {
                return option;
            }
        }
    }

    pub fn stdout(message: &str) {
        let symbol = "[MSG]".color(Color::Yellow);
        let time = format!("[{}]", Utc::now().format("%H:%M:%S")).color(Color::Green);
        println!("{} {} {}", symbol, time, message);
    }

    pub fn stdout_no_newline(message: &str) {
        let symbol = "[MSG]".color(Color::Yellow);
        let time = format!("[{}]", Utc::now().format("%H:%M:%S")).color(Color::Green);
        print!("{} {} {}", symbol, time, message);
        io::stdout().flush().unwrap();
    }

    pub fn stderr(message: &str) {
        let symbol = "[ERR]".color(Color::Red);
        let time = format!("[{}]", Utc::now().format("%H:%M:%S")).color(Color::Green);
        println!("{} {} {}", symbol, time, message);
    }

    pub fn show_sign(message: &str) {
        println!("{}", message.color(Color::Magenta));
    }

    pub fn dump(array: &[String], filename: &str) -> Result<(), io::Error> {
        let mut file = File::create(filename)?;
        for data in array {
            writeln!(file, "{}", data)?;
        }
        Ok(())
    }

    pub fn dump_json(
        array: &[(String, String, String, String)],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut json_data = std::collections::HashMap::new();

        for (index, (url, db, server, lang)) in array.iter().enumerate() {
            json_data.insert(
                index.to_string(),
                JsonVulnData {
                    url: url.clone(),
                    db: db.clone(),
                    server: server.clone(),
                    lang: lang.clone(),
                },
            );
        }

        let mut file = File::create(filename)?;
        let json_string = serde_json::to_string_pretty(&json_data)?;
        file.write_all(json_string.as_bytes())?;
        Ok(())
    }

    pub fn print_server_info(data: &[ServerData]) {
        if data.is_empty() {
            Self::stderr("No server information available");
            return;
        }

        println!("\n{}", " DOMAINS ".on_white().black().bold());
        let table = Table::new(data);
        println!("{}", table);
    }

    pub fn normal_print(data: &[(String, String)]) {
        let vuln_data: Vec<VulnData> = data
            .iter()
            .enumerate()
            .map(|(i, (url, db))| VulnData {
                index: i + 1,
                url: url.clone(),
                database: db.clone(),
            })
            .collect();

        if vuln_data.is_empty() {
            Self::stderr("No vulnerable URLs found");
            return;
        }

        println!("\n{}", " VULNERABLE URLS ".on_white().black().bold());
        let table = Table::new(&vuln_data);
        println!("{}", table);
    }

    pub fn full_print(data: &[(String, String, String, String)]) {
        let vuln_data: Vec<FullVulnData> = data
            .iter()
            .enumerate()
            .map(|(i, (url, db, server, lang))| {
                let truncated_server = if server.len() > 30 {
                    format!("{}...", &server[..27])
                } else {
                    server.clone()
                };
                let truncated_lang = if lang.len() > 30 {
                    format!("{}...", &lang[..27])
                } else {
                    lang.clone()
                };

                FullVulnData {
                    index: i + 1,
                    url: url.clone(),
                    database: db.clone(),
                    server: truncated_server,
                    language: truncated_lang,
                }
            })
            .collect();

        if vuln_data.is_empty() {
            Self::stderr("No vulnerable URLs found");
            return;
        }

        println!("\n{}", " VULNERABLE URLS ".on_white().black().bold());
        let table = Table::new(&vuln_data);
        println!("{}", table);
    }
}
