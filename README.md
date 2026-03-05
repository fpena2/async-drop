# Async Drop

Inspired by [async-dropper](https://github.com/t3hmrman/async-dropper)

## Adjustments

- Removed `async_trait` crate dependency
- Types don't have to implement `Default`
- Dropper's drop will wait until `async_drop` completes
- Only compatible with the `tokio` runtime (for now)

## Usage

```rust 
struct Thing;

impl AsyncDrop for Thing {
    fn async_drop(&mut self) -> AsyncDropFuture<'_> {
        Box::pin(async {
            tokio::time::sleep(Duration::from_secs(3)).await;
            println!("dropped");
            Ok(())
        })
    }
}

#[tokio::main]
async fn main() {
    {
        let _thing = Dropper::new(Thing);
        println!("dropping...");
    } // `_thing` is dropped here, but before that happens `async_drop()` will run to completion
}

```

## Examples

See `test` and `example` directories