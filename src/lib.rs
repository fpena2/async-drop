use std::ops::{Deref, DerefMut};

pub trait AsyncDrop {
    #[allow(async_fn_in_trait)]
    async fn async_drop(&mut self) -> Result<(), String>;
}

pub struct Dropper<T>
where
    T: AsyncDrop + 'static,
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
    T: AsyncDrop + 'static,
{
    fn drop(&mut self) {
        let Some(mut inner) = self.inner.take() else {
            return;
        };

        let async_drop_future = inner.async_drop();

        if let Err(e) = futures::executor::block_on(async_drop_future) {
            eprintln!("{}", e);
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
