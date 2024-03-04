use {
  crate::enums::event::Event,
  crossterm::{
    cursor,
    event::{
      DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event as CrosstermEvent,
      EventStream, KeyCode, KeyEventKind,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
  },
  futures::{FutureExt, StreamExt},
  ratatui::{backend::CrosstermBackend, Terminal},
  std::{io::Stderr, time::Duration},
  tokio::{
    sync::mpsc::{error::SendError, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
  },
};

/// `TuiBackend` is a struct that represents the backend for the user interface.
/// It is responsible for managing the terminal and buffering events for processing.
pub struct TuiBackend {
  /// A terminal instance that is used to render the user interface.
  pub terminal: Terminal<CrosstermBackend<Stderr>>,
  /// A join handle that represents the task for processing events.
  pub task: JoinHandle<Result<(), SendError<Event>>>,
  /// An unbounded receiver that can receive events for processing.
  pub event_rx: UnboundedReceiver<Event>,
  /// An unbounded sender that can send events for processing.
  pub event_tx: UnboundedSender<Event>,
  /// The frame rate at which the user interface should be rendered.
  pub frame_rate: f64,
  /// A boolean flag that represents whether the mouse is enabled or not.
  pub mouse: bool,
  /// A boolean flag that represents whether the paste mode is enabled or not.
  pub paste: bool,
}

impl TuiBackend {
  /// Create a new instance of the `TuiBackend` struct.
  ///
  /// # Returns
  /// * `Result<Self, io::Error>` - An Ok result containing the new instance of the `TuiBackend` struct or an error.
  pub fn new() -> Result<Self, std::io::Error> {
    let terminal = ratatui::Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let task: JoinHandle<Result<(), SendError<Event>>> = tokio::spawn(async { Err(SendError(Event::Init)) });
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let frame_rate = 60.0;
    let mouse = false;
    let paste = false;
    Ok(Self {
      terminal,
      task,
      event_rx,
      event_tx,
      frame_rate,
      mouse,
      paste,
    })
  }
  /// Enter the user interface and start processing events.
  /// This will enable the raw mode for the terminal and switch to the alternate screen.
  ///
  /// # Returns
  /// * `Result<(), io::Error>` - An Ok result or an error.
  pub fn enter(&mut self) -> Result<(), std::io::Error> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;
    if self.mouse {
      crossterm::execute!(std::io::stderr(), EnableMouseCapture)?;
    }
    if self.paste {
      crossterm::execute!(std::io::stderr(), EnableBracketedPaste)?;
    }
    self.start();
    Ok(())
  }
  /// Exit the user interface and stop processing events.
  /// This will disable the raw mode for the terminal and switch back to the main screen.
  ///
  /// # Returns
  /// * `Result<(), io::Error>` - An Ok result or an error.
  pub fn exit(&self) -> Result<(), std::io::Error> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
    if self.mouse {
      crossterm::execute!(std::io::stderr(), DisableMouseCapture)?;
    }
    if self.paste {
      crossterm::execute!(std::io::stderr(), DisableBracketedPaste)?;
    }
    Ok(())
  }
  /// Suspend the user interface and stop processing events.
  /// This will disable the raw mode for the terminal and switch back to the main screen.
  ///
  /// # Returns
  /// * `Result<(), io::Error>` - An Ok result or an error.
  pub fn suspend(&mut self) -> Result<(), std::io::Error> {
    self.exit()?;
    #[cfg(not(windows))]
    signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
    Ok(())
  }
  /// Resume the user interface and start processing events.
  ///
  /// # Returns
  /// * `Result<(), io::Error>` - An Ok result or an error.
  pub fn resume(&mut self) -> Result<(), std::io::Error> {
    self.enter()?;
    Ok(())
  }
  /// Set the frame rate at which the user interface should be rendered.
  /// The frame rate is specified in frames per second (FPS).
  /// The default frame rate is 60 FPS.
  ///
  /// # Arguments
  /// * `frame_rate` - The frame rate at which the user interface should be rendered.
  ///
  /// # Returns
  /// * `Self` - The modified instance of the `TuiBackend` struct.
  pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
    self.frame_rate = frame_rate;
    self
  }
  /// Enable or disable the mouse for the user interface.

  /// By default, the mouse is disabled.
  ///
  /// # Arguments
  /// * `mouse` - A boolean flag that represents whether the mouse is enabled or not.
  ///
  /// # Returns
  /// * `Self` - The modified instance of the `TuiBackend` struct.
  pub fn with_mouse(mut self, mouse: bool) -> Self {
    self.mouse = mouse;
    self
  }
  /// Enable or disable the paste mode for the user interface.
  /// By default, the paste mode is disabled.
  ///
  /// # Arguments
  /// * `paste` - A boolean flag that represents whether the paste mode is enabled or not.
  ///
  /// # Returns
  /// * `Self` - The modified instance of the `TuiBackend` struct.
  pub fn with_paste(mut self, paste: bool) -> Self {
    self.paste = paste;
    self
  }
  /// Send an event asynchronously for processing.
  /// This will pop from the event queue the first event that is ready and return it.
  /// If no event is available, this will sleep until an event is available.
  ///
  /// # Returns
  /// * `Option<Event>` - An optional event that is ready for processing.
  pub async fn next(&mut self) -> Option<Event> {
    self.event_rx.recv().await
  }

  // ==============================
  // Private functions
  // ==============================

  /// Start processing events asynchronously.
  /// This will spawn a new task that will process events.
  /// The task will listen for events from the terminal and send them to the event queue for processing.
  fn start(&mut self) {
    let _event_tx = self.event_tx.clone();
    let render_delay = Duration::from_secs_f64(1.0 / self.frame_rate);

    self.task = tokio::spawn(async move {
      let mut reader = EventStream::new();
      let mut render_interval = tokio::time::interval(render_delay);

      _event_tx.send(Event::Init)?;
      loop {
        let crossterm_event = reader.next().fuse();
        let render_tick = render_interval.tick();

        tokio::select! {
          maybe_event = crossterm_event => {
            match maybe_event {
              Some(Ok(event)) => {
                match event {
                  CrosstermEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                      if key.code == KeyCode::Char('q') {
                        _event_tx.send(Event::Quit)?;
                      } else {
                        _event_tx.send(Event::Key(key))?;
                      }
                    }
                  },
                  CrosstermEvent::Mouse(mouse) => {
                    _event_tx.send(Event::Mouse(mouse))?;
                  },
                  CrosstermEvent::Resize(width, height) => {
                    _event_tx.send(Event::Resize(width, height))?;
                  },
                  CrosstermEvent::FocusLost => {} // TODO: handle focus lost
                  CrosstermEvent::FocusGained => {} // TODO: handle focus gained
                  _ => unimplemented!()
                }
              },
              _ => unimplemented!()
            }
          },
          _ = render_tick => {
            _event_tx.send(Event::Render)?;
          }
        }
      }
    });
  }
}