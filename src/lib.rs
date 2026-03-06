use std::ops::{Deref, DerefMut};

pub trait AsyncDrop {
    #[allow(async_fn_in_trait)]
    async fn async_drop(&mut self) -> Result<(), String>;
}

pub struct Dropper<T>
where
    T: AsyncDrop + Send + 'static,
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
    T: AsyncDrop + Send + 'static,
{
    fn drop(&mut self) {
        let Some(mut inner) = self.inner.take() else {
            return;
        };

        // Spawn a dedicated thread with its own tokio runtime so we don't
        // deadlock when dropped inside a single-threaded tokio executor.
        std::thread::scope(|s| {
            s.spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build async-drop runtime");
                if let Err(e) = rt.block_on(inner.async_drop()) {
                    eprintln!("{}", e);
                }
            });
        });
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
