axum extractor for [bebop](https://github.com/6over3/bebop) records

### Example

```rust

// NOTE: MyRecord must be an owned record
pub async fn handler(Bebop(record): Bebop<owned::MyRecord>) -> impl IntoResponse {
    // do something with record
} 
```
