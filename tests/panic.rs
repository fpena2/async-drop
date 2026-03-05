use async_drop::{AsyncDrop, AsyncDropFuture, Dropper};

struct AsyncThing;

impl AsyncDrop for AsyncThing {
    fn async_drop(&mut self) -> AsyncDropFuture<'_> {
        Box::pin(async {
            panic!("Something happened");
        })
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn dropper_calls_async_drop_which_panics() {
    {
        let thing = AsyncThing;
        let _example_obj = Dropper::new(thing);

        // You can set a no-op hook temporarily to supress the panic printout
        // std::panic::set_hook(Box::new(|_| {}));

        println!("about to drop...");
    }
}
