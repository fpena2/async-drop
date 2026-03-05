use std::ops::{Deref, DerefMut};

pub type AsyncDropFuture<'a> =
    std::pin::Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

pub trait AsyncDrop {
    fn async_drop(&mut self) -> AsyncDropFuture<'_> {
        Box::pin(async { Ok(()) })
    }
}

#[derive(Default)]
pub struct Dropper<T>
where
    T: AsyncDrop + Default + Send + Sync + 'static,
{
    dropped: bool,
    inner: T,
}

impl<T> Dropper<T>
where
    T: AsyncDrop + Default + Send + Sync + 'static,
{
    pub fn new(inner: T) -> Self {
        Self {
            dropped: false,
            inner,
        }
    }
}

impl<T> Drop for Dropper<T>
where
    T: AsyncDrop + Default + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if self.dropped {
            return;
        }

        let mut this = Dropper::default();
        std::mem::swap(&mut this, self);
        this.dropped = true;

        let (done_trigger, done_signal) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                futures::executor::block_on(this.inner.async_drop())
            }));

            let flattened = match result {
                Ok(inner) => inner.map_err(|e| format!("Async drop error: {}", e)),
                Err(_) => Err("Task panicked".to_owned()),
            };

            let _ = done_trigger.send(flattened);
        });

        match futures::executor::block_on(done_signal) {
            Ok(result) => {
                if let Err(e) = result {
                    eprintln!("{}", e);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}

impl<T> Deref for Dropper<T>
where
    T: AsyncDrop + Default + Send + Sync + 'static,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Dropper<T>
where
    T: AsyncDrop + Default + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
