use core::iter::FromIterator;
use core::mem;
use core::pin::Pin;
use alloc::vec::Vec;
use futures_core::future::Future;
use futures_core::task::{Context, Poll};

/// Future for the [`select_all`] function.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct SelectAll<Fut> {
    inner: Vec<Fut>,
}

impl<Fut: Unpin> Unpin for SelectAll<Fut> {}

/// Creates a new future which will select over a list of futures.
///
/// The returned future will wait for any future within `iter` to be ready. Upon
/// completion the item resolved will be returned, along with the index of the
/// future that was ready and the list of all the remaining futures.
///
/// # Panics
///
/// This function will panic if the iterator specified contains no items.
pub fn select_all<I>(iter: I) -> SelectAll<I::Item>
    where I: IntoIterator,
          I::Item: Future + Unpin,
{
    let ret = SelectAll {
        inner: iter.into_iter().collect()
    };
    assert!(!ret.inner.is_empty());
    ret
}

impl<Fut: Future + Unpin> Future for SelectAll<Fut> {
    type Output = (Fut::Output, usize, Vec<Fut>);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let item = self.inner.iter_mut().enumerate().find_map(|(i, f)| {
            match Pin::new(f).poll(cx) {
                Poll::Pending => None,
                Poll::Ready(e) => Some((i, e)),
            }
        });
        match item {
            Some((idx, res)) => {
                self.inner.remove(idx);
                let rest = mem::replace(&mut self.inner, Vec::new());
                Poll::Ready((res, idx, rest))
            }
            None => Poll::Pending,
        }
    }
}

impl<Fut: Future + Unpin> FromIterator<Fut> for SelectAll<Fut> {
    fn from_iter<T: IntoIterator<Item = Fut>>(iter: T) -> Self {
        select_all(iter)
    }
}
