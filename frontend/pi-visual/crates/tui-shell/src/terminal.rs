//! One terminal/input owner with bounded backpressure and reversible mode setup.
use crossterm::{
    cursor::Show,
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
pub struct TerminalGuard {
    pub terminal: ratatui::DefaultTerminal,
    mouse: bool,
    paste: bool,
    keyboard: bool,
}
impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        let terminal = match ratatui::try_init() {
            Ok(t) => t,
            Err(e) => {
                ratatui::restore();
                return Err(e);
            }
        };
        let mut guard = Self {
            terminal,
            mouse: false,
            paste: false,
            keyboard: false,
        };
        execute!(io::stdout(), EnableMouseCapture)?;
        guard.mouse = true;
        execute!(io::stdout(), EnableBracketedPaste)?;
        guard.paste = true;
        execute!(
            io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
        guard.keyboard = true;
        Ok(guard)
    }
}
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.keyboard {
            let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
        }
        if self.paste {
            let _ = execute!(io::stdout(), DisableBracketedPaste);
        }
        if self.mouse {
            let _ = execute!(io::stdout(), DisableMouseCapture);
        }
        let _ = execute!(io::stdout(), Show);
        ratatui::restore();
    }
}
pub struct Input {
    pub events: tokio::sync::mpsc::Receiver<io::Result<Event>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Input {
    pub fn start() -> Self {
        let (tx, events) = tokio::sync::mpsc::channel(64);
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let thread = std::thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                match event::poll(Duration::from_millis(100)) {
                    Ok(false) => continue,
                    Ok(true) => {
                        let event = event::read();
                        let failed = event.is_err();
                        if tx.blocking_send(event).is_err() || failed {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.blocking_send(Err(e));
                        break;
                    }
                }
            }
        });
        Self {
            events,
            stop,
            thread: Some(thread),
        }
    }
    pub async fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.events.close();
        if let Some(thread) = self.thread.take() {
            let _ = tokio::task::spawn_blocking(move || thread.join()).await;
        }
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.events.close();
    }
}
#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires PTY; invoked by scripts/test-transcript-pty.py"]
    fn panic_restores_modes() {
        let result = std::panic::catch_unwind(|| {
            let _guard = super::TerminalGuard::enter().unwrap();
            panic!("controlled terminal cleanup test")
        });
        assert!(result.is_err());
        assert!(!crossterm::terminal::is_raw_mode_enabled().unwrap());
    }
}
