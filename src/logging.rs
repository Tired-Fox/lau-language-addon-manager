use std::{io::{stderr, stdout, Write}, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc}, thread::JoinHandle, time::Duration};

pub mod colors {
    pub use owo_colors::*;
    pub use owo_colors::colors::*;
}

pub use colors::OwoColorize;

pub trait Logger {
    fn update(&mut self, log: impl std::fmt::Display);
    fn error(&mut self, log: impl std::fmt::Display);
    fn success(&mut self, log: impl std::fmt::Display);
    fn warning(&mut self, log: impl std::fmt::Display);
    fn finish(&mut self);
}

pub trait OrLog<L: Logger, O = ()> {
    /// Consume the value and log
    fn log(self, logger: &mut L);
    /// Same as [`log`][crate::logging::OrLog::log] but takes in a custom message
    fn log_with(self, logger: &mut L, message: impl std::fmt::Display);
    /// Consume the value and log
    ///
    /// If the value is empty, error, etc. `Other` will be returned.
    /// Works similar to `unwrap_or`.
    fn log_or(self, logger: &mut L, other: O) -> O;
    /// Same as [`log_or`][crate::logging::OrLog::log_or] but takes in a custom message
    fn log_with_or(self, logger: &mut L, message: impl std::fmt::Display, other: O) -> O;
}

impl<O, E: std::fmt::Display, L: Logger> OrLog<L, O> for Result<O, E> {
    fn log(self, logger: &mut L) {
        if let Err(err) = self {
           logger.error(err);
        }
    }

    fn log_with(self, logger: &mut L, message: impl std::fmt::Display) {
        if self.is_err() {
           logger.error(message);
        }
    }

    fn log_or(self, logger: &mut L, other: O) -> O {
        match self {
            Ok(value) => value,
            Err(err) => {
                logger.error(err);
                other
            }
        }
    }

    fn log_with_or(self, logger: &mut L, message: impl std::fmt::Display, other: O) -> O {
        match self {
            Ok(value) => value,
            Err(_) => {
                logger.error(message);
                other
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stream {
    Stdout,
    Stderr,
}
impl Stream {
    pub fn get(&self) -> Box<dyn Write + Send + Sync> {
        match self {
            Self::Stdout => Box::new(stdout()),
            Self::Stderr => Box::new(stderr()),
        }
    }
}
impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdout => stdout().write(buf),
            Self::Stderr => stderr().write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdout => stdout().flush(),
            Self::Stderr => stderr().flush(),
        } 
    }
}

#[macro_export]
macro_rules! frames {
    ([ $($frame: expr),* $(,)? ], $interval: expr) => {
        Vec::from([
            $($crate::logging::Frame::new($frame, $interval),)*
        ])
    };
    ([ $($frame: expr),* $(,)? ], $interval: expr, $color: ty) => {
        Vec::from([
            $($crate::logging::Frame::new_with_color::<$color>($frame, $interval),)*
        ])
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    text: String,
    interval: Duration
}

impl Frame {
    pub fn new(text: impl std::fmt::Display, interval: Duration) -> Self {
        Self { text: text.to_string(), interval }
    }

    pub fn new_with_color<C: colors::Color>(text: impl std::fmt::Display, interval: Duration) -> Self {
        Self { text: text.to_string().fg::<C>().to_string(), interval }
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug)]
pub struct Spinner {
    channel: Sender<Option<String>>,
    handle: Option<JoinHandle<()>>,
    spinning: Arc<AtomicBool>,

    stream: Stream,
}

impl Spinner {
    /// Create a new spinner
    ///
    /// The spinner creates a thread and start immediatly. However, it will not render until it is
    /// updated with a message to display.
    pub fn new(mut target: Stream, frames: Vec<Frame>) -> Self {
        let (s, r) = std::sync::mpsc::channel::<Option<String>>();

        let sp = Arc::new(AtomicBool::new(true));

        let spinning = sp.clone();
        let handle = std::thread::spawn(move || {
            let mut message: Option<String> = None;
            let frames = frames.iter().cycle().take_while(|_| spinning.load(Ordering::Relaxed));

            for frame in frames {
                if let Ok(msg) = r.try_recv() {
                    message = msg;
                }

                let fout = match message.as_deref() {
                    Some(msg) => format!("{frame} {msg}"),
                    None => String::new(),
                };

                let _ = write!(target, "\r\x1b[0K{fout}");
                let _ = target.flush();

                std::thread::sleep(frame.interval);
            }

            let _ = write!(target, "\r\x1b[0K");
            spinning.store(false, Ordering::Relaxed);
        });

        Self {
            channel: s,
            handle: Some(handle),
            spinning: sp,

            stream: target
        } 
    }

    /// Check if the spinner is running
    pub fn is_spinning(&self) -> bool {
        self.spinning.load(Ordering::Relaxed)
    }

    /// Update the message of the spinner line
    pub fn update_message(&self, msg: impl std::fmt::Display) {
        let _ = self.channel.send(Some(msg.to_string()));
    }

    /// Clear the spinner line
    ///
    /// The spinner will keep running, it will just not display anything since there
    /// is no message to display.
    pub fn clear(&self) {
        let _ = self.channel.send(None);
    }

    /// Stop the spinner and wait for it to exit
    pub fn stop(&mut self) {
        let _ = self.channel.send(None);
        self.spinning.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            if !handle.is_finished() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Logger for Spinner {
    fn update(&mut self, log: impl std::fmt::Display) {
        self.update_message(log.to_string());
    }

    fn error(&mut self, log: impl std::fmt::Display) {
        let _ = writeln!(self.stream, "\r\x1b[0K{} {}", "✕".red().bold(), log);
    }

    fn success(&mut self, log: impl std::fmt::Display) {
        let _ = writeln!(self.stream, "\r\x1b[0K{} {}", "✓".green().bold(), log);
    }

    fn warning(&mut self, log: impl std::fmt::Display) {
        let _ = writeln!(self.stream, "\r\x1b[0K{} {}", "⚠".yellow().bold(), log);
    }

    fn finish(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn spinner() {
        let _ = Vec::from([
            Frame::new_with_color::<colors::xterm::Blue>("⠋", Duration::from_millis(80)),
        ]);

        let mut spinner = Spinner::new(Stream::Stdout, frames!(["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], Duration::from_millis(80), colors::xterm::AeroBlue));
        assert!(spinner.is_spinning());

        spinner.update("First message");

        std::thread::sleep(Duration::from_secs(3));
        spinner.update("Second message");

        std::thread::sleep(Duration::from_secs(1));
        Logger::success(&mut spinner, "test");

        std::thread::sleep(Duration::from_secs(1));
        Logger::warning(&mut spinner, "test");
        spinner.update("Hello, world!");

        std::thread::sleep(Duration::from_secs(2));
        spinner.stop();

        assert!(!spinner.is_spinning());
    }
}
