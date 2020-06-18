use std::io::{self, *};
use std::sync::mpsc::*;
use std::thread;
use std::time::*;
use termion::input::*;
use termion::raw::*;
use tui::backend::*;

pub type Backend = TermionBackend<RawTerminal<Stdout>>;
pub type Frame<'a> = tui::Frame<'a, Backend>;
pub type Terminal = tui::Terminal<Backend>;

#[derive(PartialEq)]
pub enum Event {
    Termion(termion::event::Event),
    Tick,
    Exit,
}

pub struct Ui {
    terminal: Terminal,
    events: Receiver<Event>,
}

impl Ui {
    pub fn new(tick_rate: Duration) -> io::Result<Self> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear().ok();
        terminal.hide_cursor().ok();

        Ok(Self {
            terminal,
            events: event_stream(tick_rate),
        })
    }

    pub fn next_event(&self) -> Event {
        self.events.recv().unwrap_or(Event::Exit)
    }

    pub fn terminal(&mut self) -> &mut Terminal {
        &mut self.terminal
    }
}

fn event_stream(tick_rate: Duration) -> Receiver<Event> {
    let (tx, rx) = sync_channel(25);

    std::thread::spawn({
        let tx = tx.clone();

        move || {
            let stdin = std::io::stdin();
            let locked_stdin = stdin.lock();
            let mut events = locked_stdin.events();

            while let Ok(event) = events.next().transpose() {
                if let Some(event) = event {
                    if tx.send(Event::Termion(event)).is_err() {
                        return;
                    }
                }
            }

            tx.send(Event::Exit).ok();
        }
    });

    std::thread::spawn({
        let tx = tx.clone();

        move || {
            while let Ok(_) = tx.send(Event::Tick) {
                thread::sleep(tick_rate);
            }
        }
    });

    rx
}

pub trait View {
    type Result;
    fn update(&mut self, event: Event, ui: &mut Ui) -> Option<Self::Result>;
    fn render(&mut self, frame: &mut Frame);
    fn run(mut self, ui: &mut Ui) -> Self::Result
    where
        Self: Sized,
    {
        loop {
            let event = ui.next_event();
            
            if event == Event::Exit {
                std::process::exit(0);
            }

            if let Some(result) = self.update(event, ui) {
                return result;
            }

            ui.terminal().draw(|mut frame| self.render(&mut frame)).ok();
        }
    }
}
