# A session with cookies

## usage
```rust
use asession::SessionBuilder;

#[tokio::main]
async fn main() {
    let session: Session = SessionBuilder::new()
        .cookies_store_into("cookies.json".into())
        .build().unwrap();

    let res = session.post("https://www.example.com")
        .form(&[("key", "value")])
        .send()
        .await
        .unwrap();
}
```