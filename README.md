# asession 
a user-friendly Client wrapper, which automatically handles cookies and load/store cookies from/to the specified path.

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