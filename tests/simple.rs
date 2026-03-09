use async_drop::{AsyncDrop, AsyncDropFuture, Dropper};
use std::time::Duration;

struct AsyncThing(String);

impl AsyncDrop for AsyncThing {
    fn async_drop(&mut self) -> AsyncDropFuture<'_> {
        Box::pin(async {
            println!("async dropping [{}]!", self.0);
            println!("sleeping for 2 seconds");
            tokio::time::sleep(Duration::from_secs(2)).await;
            println!("done sleeping");
            println!("async dropped [{}]!", self.0);
            Ok(())
        })
    }
}

#[tokio::test]
async fn dropper_calls_async_drop_when_dropped() {
    {
        let thing = AsyncThing(String::from("test"));
        let _dropper = Dropper::new(thing);
        println!("here comes the (async) drop");
    } // dropper is dropped here, but before that happens AsyncThing's `async_drop` will be called 
}

#[tokio::test]
async fn dropper_has_deref_and_deref_mut_which_expose_the_inner_struct() {
    {
        let thing = Dropper::new(AsyncThing(String::from("test")));
        // Dropper is a wrapper, we have access to AsyncThing's fields via deref and deref_mut
        let _str = &thing.0;
    }
}
