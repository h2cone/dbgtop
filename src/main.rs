use std::{
    env,
    fs::{self, File},
    io::Write,
    time::Instant,
};

#[tokio::main]
async fn main() {
    let now = Instant::now();
    if let Err(e) = execute().await {
        eprintln!("Failed to execute, err={}", e);
    }
    println!("Time elapsed in execute() is: {:?}", now.elapsed());
}

async fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let conf_path = env::args().nth(1).unwrap_or("conf.toml".to_string());
    let conf_str = std::fs::read_to_string(conf_path)?;
    let conf = &conf_str.parse::<toml::Value>()?["douban-game"];

    let mut more = conf["more"].as_str().unwrap_or("").to_string();
    let sink_type: &str = conf["sink_type"].as_str().unwrap();
    let mut sink: Box<dyn Sink> = match sink_type {
        "file" => Box::new(FileSink::new(&conf)),
        _ => panic!("Unsupported sink type"),
    };

    let url = conf["url"].as_str().unwrap();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "*".parse()?);
    let client = reqwest::Client::new();

    let idx = "more";
    let null = serde_json::Value::Null.to_string();
    while more != null {
        let builder = client
            .get(url)
            .query(&[("sort", "rating"), ("more", &more)])
            .headers(headers.clone());

        let resp = builder.send().await?;
        let body = resp.json::<serde_json::Value>().await?;

        sink.consume(&body);

        more = body
            .get(idx)
            .unwrap_or(&serde_json::Value::Null)
            .to_string();

        println!("?more={}", more);
    }
    Ok(())
}

trait Sink {
    fn consume(&mut self, data: &serde_json::Value);
}

struct FileSink {
    file: File,
}

impl FileSink {
    fn new(conf: &toml::Value) -> Self {
        let path = conf["sink_to"].as_str().unwrap();

        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        FileSink { file }
    }
}

impl Sink for FileSink {
    fn consume(&mut self, body: &serde_json::Value) {
        if let Err(e) = serde_json::to_writer(&self.file, &body) {
            eprintln!("Failed to write to file, err={}", e);
        } else {
            writeln!(self.file).unwrap();
        }
    }
}
