use url::Url;

fn main() {
    match Url::parse("https://") {
        Ok(url) => {
            println!("Parsed successfully: {:?}", url);
            println!("Host: {:?}", url.host_str());
        }
        Err(e) => {
            println!("Parse failed: {}", e);
        }
    }
}
