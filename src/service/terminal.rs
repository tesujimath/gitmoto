use crossterm::event::{KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use std::future::Future;
use tokio::{select, sync::mpsc};

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Terminal event handler.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Service {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    /// Whether the channel is closed.
    receiver_closed: bool,
    /// Event handler thread.
    handler: tokio::task::JoinHandle<()>,
}

impl Default for Service {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            loop {
                let crossterm_event = reader.next().fuse();
                select! {
                  _ = _sender.closed() => {
                    break;
                  }
                  Some(Ok(evt)) = crossterm_event => {
                      use crossterm::event::Event::*;
                    match evt {
                      Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                          _sender.send(Event::Key(key)).unwrap();
                        }
                      },
                      Mouse(mouse) => {
                        _sender.send(Event::Mouse(mouse)).unwrap();
                      },
                      Resize(x, y) => {
                        _sender.send(Event::Resize(x, y)).unwrap();
                      },
                      FocusLost => {
                      },
                      FocusGained => {
                      },
                      Paste(_) => {
                      },
                    }
                  }
                };
            }
        });
        Self {
            sender,
            receiver,
            receiver_closed: false,
            handler,
        }
    }
}

impl Service {
    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    ///
    /// None is returned when the terminal is closed.
    pub fn recv_event(&mut self) -> impl Future<Output = Option<Event>> + '_ {
        self.receiver.recv()
    }

    pub async fn handle<F>(&mut self, ev: Event, key_handler: F) -> bool
    where
        F: FnOnce(KeyEvent) -> bool,
    {
        use Event::*;
        let mut quit = false;
        match ev {
            Key(key_event) => quit = key_handler(key_event),
            Mouse(_) => {}
            Resize(_, _) => {}
        }

        quit
    }
}
