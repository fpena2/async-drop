use std::ops::{Deref, DerefMut};

pub type AsyncDropFuture<'a> =
    std::pin::Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

pub trait AsyncDrop {
    fn async_drop(&mut self) -> AsyncDropFuture<'_> {
        Box::pin(async { Ok(()) })
    }
}

pub struct Dropper<T>
where
    T: AsyncDrop + Send + Sync + 'static,
{
    inner: Option<T>,
}

impl<T> Dropper<T>
where
    T: AsyncDrop + Send + Sync + 'static,
{
    pub fn new(inner: T) -> Self {
        Self { inner: Some(inner) }
    }
}

impl<T> Drop for Dropper<T>
where
    T: AsyncDrop + Send + Sync + 'static,
{
    fn drop(&mut self) {
        let Some(mut inner) = self.inner.take() else {
            return;
        };

        let (done_trigger, done_signal) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                futures::executor::block_on(inner.async_drop())
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
    T: AsyncDrop + Send + Sync + 'static,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
            .as_ref()
            .expect("Dropper value has already been dropped")
    }
}

impl<T> DerefMut for Dropper<T>
where
    T: AsyncDrop + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
            .as_mut()
            .expect("Dropper value has already been dropped")
    }
}
