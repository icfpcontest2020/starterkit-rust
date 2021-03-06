use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use futures_io::{AsyncBufRead, AsyncWrite};
use std::io;
use std::pin::Pin;
use pin_project::{pin_project, project};

/// Creates a future which copies all the bytes from one object to another.
///
/// The returned future will copy all the bytes read from this `AsyncBufRead` into the
/// `writer` specified. This future will only complete once the `reader` has hit
/// EOF and all bytes have been written to and flushed from the `writer`
/// provided.
///
/// On success the number of bytes is returned.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures::io::{self, AsyncWriteExt, Cursor};
///
/// let reader = Cursor::new([1, 2, 3, 4]);
/// let mut writer = Cursor::new(vec![0u8; 5]);
///
/// let bytes = io::copy_buf(reader, &mut writer).await?;
/// writer.close().await?;
///
/// assert_eq!(bytes, 4);
/// assert_eq!(writer.into_inner(), [1, 2, 3, 4, 0]);
/// # Ok::<(), Box<dyn std::error::Error>>(()) }).unwrap();
/// ```
pub fn copy_buf<R, W>(reader: R, writer: &mut W) -> CopyBuf<'_, R, W>
where
    R: AsyncBufRead,
    W: AsyncWrite + Unpin + ?Sized,
{
    CopyBuf {
        reader,
        writer,
        amt: 0,
    }
}

/// Future for the [`copy_buf()`] function.
#[pin_project]
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct CopyBuf<'a, R, W: ?Sized> {
    #[pin]
    reader: R,
    writer: &'a mut W,
    amt: u64,
}

impl<R, W> Future for CopyBuf<'_, R, W>
    where R: AsyncBufRead,
          W: AsyncWrite + Unpin + ?Sized,
{
    type Output = io::Result<u64>;

    #[project]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[project]
        let CopyBuf { mut reader, mut writer, amt } = self.project();
        loop {
            let buffer = ready!(reader.as_mut().poll_fill_buf(cx))?;
            if buffer.is_empty() {
                ready!(Pin::new(&mut writer).poll_flush(cx))?;
                return Poll::Ready(Ok(*amt));
            }

            let i = ready!(Pin::new(&mut writer).poll_write(cx, buffer))?;
            if i == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()))
            }
            *amt += i as u64;
            reader.as_mut().consume(i);
        }
    }
}
