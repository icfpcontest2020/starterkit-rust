use core::fmt;
use core::pin::Pin;
use futures_core::stream::{FusedStream, Stream, TryStream};
use futures_core::task::{Context, Poll};
#[cfg(feature = "sink")]
use futures_sink::Sink;
use pin_utils::{unsafe_pinned, unsafe_unpinned};

/// Stream for the [`map_err`](super::TryStreamExt::map_err) method.
#[must_use = "streams do nothing unless polled"]
pub struct MapErr<St, F> {
    stream: St,
    f: F,
}

impl<St: Unpin, F> Unpin for MapErr<St, F> {}

impl<St, F> fmt::Debug for MapErr<St, F>
where
    St: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapErr")
            .field("stream", &self.stream)
            .finish()
    }
}

impl<St, F> MapErr<St, F> {
    unsafe_pinned!(stream: St);
    unsafe_unpinned!(f: F);

    /// Creates a new MapErr.
    pub(super) fn new(stream: St, f: F) -> Self {
        MapErr { stream, f }
    }

    /// Acquires a reference to the underlying stream that this combinator is
    /// pulling from.
    pub fn get_ref(&self) -> &St {
        &self.stream
    }

    /// Acquires a mutable reference to the underlying stream that this
    /// combinator is pulling from.
    ///
    /// Note that care must be taken to avoid tampering with the state of the
    /// stream which may otherwise confuse this combinator.
    pub fn get_mut(&mut self) -> &mut St {
        &mut self.stream
    }

    /// Acquires a pinned mutable reference to the underlying stream that this
    /// combinator is pulling from.
    ///
    /// Note that care must be taken to avoid tampering with the state of the
    /// stream which may otherwise confuse this combinator.
    pub fn get_pin_mut<'a>(self: Pin<&'a mut Self>) -> Pin<&'a mut St> {
        self.stream()
    }

    /// Consumes this combinator, returning the underlying stream.
    ///
    /// Note that this may discard intermediate state of this combinator, so
    /// care should be taken to avoid losing resources when this is called.
    pub fn into_inner(self) -> St {
        self.stream
    }
}

impl<St: FusedStream, F> FusedStream for MapErr<St, F> {
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St, F, E> Stream for MapErr<St, F>
where
    St: TryStream,
    F: FnMut(St::Error) -> E,
{
    type Item = Result<St::Ok, E>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.as_mut()
            .stream()
            .try_poll_next(cx)
            .map(|opt| opt.map(|res| res.map_err(|e| self.as_mut().f()(e))))
    }
}

// Forwarding impl of Sink from the underlying stream
#[cfg(feature = "sink")]
impl<S, F, Item> Sink<Item> for MapErr<S, F>
where
    S: Sink<Item>,
{
    type Error = S::Error;

    delegate_sink!(stream, Item);
}