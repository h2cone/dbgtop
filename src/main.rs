use std::{env, fs, io::Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse conf.toml
    let conf_path = env::args().nth(1).unwrap_or("conf.toml".to_string());
    let conf_str = std::fs::read_to_string(conf_path)?;
    let conf = conf_str.parse::<toml::Value>()?;

    let url = conf["douban-game"]["search_url"].as_str().unwrap();
    let result = conf["douban-game"]["search_result_file"].as_str().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "*".parse()?);

    let client = reqwest::Client::new();

    let now = chrono::Local::now();
    println!("Start to fetch data at {}", now);

    let idx = "more";
    let mut more = "".to_string();
    while more != "Null" {
        let builder = client
            .get(url)
            .query(&[("sort", "rating"), ("more", &more)])
            .headers(headers.clone());

        let resp = builder.send().await?;
        let body = resp.json::<serde_json::Value>().await?;

        // append to result file
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(result)?;

        println!("Writing more={}", more);

        if let Err(e) = serde_json::to_writer(&file, &body) {
            eprintln!("Failed to write more={}, err={}", more, e);
        }
        writeln!(file)?;

        more = body
            .get(idx)
            .unwrap_or(&serde_json::Value::Null)
            .to_string();
    }

    println!(
        "It took {} seconds to fetch data",
        chrono::Local::now()
            .signed_duration_since(now)
            .num_seconds()
    );

    Ok(())
}
