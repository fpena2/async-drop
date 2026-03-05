use async_drop::{AsyncDrop, AsyncDropFuture, Dropper};
use std::time::Duration;

#[derive(Default)]
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
