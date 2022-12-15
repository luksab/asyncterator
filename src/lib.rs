use std::thread::{Scope, ScopedJoinHandle};

/// A Asyncterator of type T
///
/// The Asyncterator is a wrapper around an Iterator of type T.
/// It spawns a thread, which executes the next() method of the inner iterator.
/// The next() method of the Asyncterator returns the cached value of the inner iterator.
///
/// It requires scoped threads.
pub struct Asyncterator<'a, T>
where
    T: Iterator + std::marker::Send,
{
    thread: ScopedJoinHandle<'a, T>,
    receiver: std::sync::mpsc::Receiver<Option<T::Item>>,
    sender: std::sync::mpsc::Sender<()>,
}

impl<'a, T> Asyncterator<'a, T>
where
    T: Iterator + std::marker::Send + 'a,
    <T as std::iter::Iterator>::Item: std::marker::Send,
{
    /// Create a new Asyncterator
    pub fn new(inner: T, scope: &'a Scope<'a, '_>) -> Self {
        let thread = spaw_thread(inner, scope);
        Self {
            thread: thread.0,
            receiver: thread.1,
            sender: thread.2,
        }
    }

    pub fn get_inner(self) -> T {
        self.thread.join().unwrap()
    }
}

fn spaw_thread<'a, T>(
    inner: T,
    scope: &'a Scope<'a, '_>,
) -> (
    ScopedJoinHandle<'a, T>,
    std::sync::mpsc::Receiver<Option<T::Item>>,
    std::sync::mpsc::Sender<()>,
)
where
    T: Iterator + std::marker::Send + 'a,
    <T as std::iter::Iterator>::Item: std::marker::Send,
{
    let (main_sender, thead_receiver) = std::sync::mpsc::channel();
    let (thead_sender, main_receiver) = std::sync::mpsc::channel();
    let thread = scope.spawn(move || {
        let mut inner = inner;
        // wait for messages on thead_receiver
        for _ in thead_receiver {
            let item = inner.next();
            thead_sender.send(item).unwrap();
        }
        inner
    });
    // initial send
    main_sender.send(()).unwrap();
    (thread, main_receiver, main_sender)
}

impl<'a, T> Iterator for Asyncterator<'a, T>
where
    T: Iterator + std::marker::Send,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.sender.send(()).unwrap();
        self.receiver.recv().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        std::thread::scope(|s| {
            let mut iter = Asyncterator::new(0..10, s);
            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next(), Some(1));
            assert_eq!(iter.next(), Some(2));
            assert_eq!(iter.next(), Some(3));
            assert_eq!(iter.next(), Some(4));
            assert_eq!(iter.next(), Some(5));
            assert_eq!(iter.next(), Some(6));
            assert_eq!(iter.next(), Some(7));
            assert_eq!(iter.next(), Some(8));
            assert_eq!(iter.next(), Some(9));
            assert_eq!(iter.next(), None);
        })
    }
}
