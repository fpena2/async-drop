use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

pub type AsyncDropFuture<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

pub trait AsyncDrop {
    fn async_drop(&mut self) -> AsyncDropFuture<'_>;
}

pub struct Dropper<T>
where
    T: AsyncDrop,
{
    inner: Option<T>,
}

impl<T> Dropper<T>
where
    T: AsyncDrop,
{
    pub fn new(inner: T) -> Self {
        Self { inner: Some(inner) }
    }
}

impl<T> Drop for Dropper<T>
where
    T: AsyncDrop,
{
    fn drop(&mut self) {
        let Some(mut inner) = self.inner.take() else {
            return;
        };

        let future = inner.async_drop();

        // Spawn a dedicated thread with its own tokio runtime so we don't
        // deadlock when dropped inside a single-threaded tokio executor.
        std::thread::scope(|s| {
            s.spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build async-drop runtime");
                if let Err(e) = rt.block_on(future) {
                    eprintln!("{}", e);
                }
            });
        });
    }
}

impl<T> Deref for Dropper<T>
where
    T: AsyncDrop + 'static,
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
