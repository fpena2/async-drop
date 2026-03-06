use async_drop::{AsyncDrop, Dropper};

struct AsyncThing;

impl AsyncDrop for AsyncThing {
    async fn async_drop(&mut self) -> Result<(), String> {
        panic!("Something happened");
    }
}

#[tokio::test]
#[should_panic]
async fn dropper_calls_async_drop_which_panics() {
    {
        let thing = AsyncThing;
        let _example_obj = Dropper::new(thing);

        // You can set a no-op hook temporarily to supress the panic printout
        // std::panic::set_hook(Box::new(|_| {}));

        println!("about to drop...");
    }
}
