use std::time::*;
use std::io;
use termion::event::*;
use tui::style::*;
use tui::widgets::*;

const FPS_30: Duration = Duration::from_millis(33);

mod ui;
use ui::{Ui, Frame, View};

fn main() -> Result<(), io::Error> {
    let ref mut ui = Ui::new(FPS_30)?;

    GroupList::new().run(ui);

    ui.terminal().clear().ok();

    Ok(())
}

struct GroupList {
    groups: Vec<String>,
    index: ListState,
    frame_count: usize,
}

impl GroupList {
    pub fn new() -> Self {
        Self::with_groups(vec![
            "Entry 1".into(),
            "Entry 2".into(),
            "Entry 3".into(),
            "Entry 4".into(),
            "Entry 5".into(),
            "Entry 6".into(),
            "Entry 7".into(),
            "Entry 8".into(),
        ])
    }

    pub fn with_groups(groups: Vec<String>) -> Self {
        let mut index = ListState::default();
        index.select(Some(0));

        Self {
            groups,
            index,
            frame_count: 0,
        }
    }
}

impl View for GroupList {
    type Result = ();

    fn update(&mut self, event: ui::Event, ui: &mut Ui) -> Option<()> {
        match event {
            ui::Event::Termion(Event::Key(key)) =>  match key {
                Key::Up => {
                    let index = self.index.selected().unwrap_or(0);

                    if index > 0 {
                        let new_index = index - 1;
                        self.index.select(Some(new_index));
                    }
                },
                Key::Down => {
                    let index = self.index.selected().unwrap_or(0);

                    if self.groups.len() > 0 {
                        let new_index = (index + 1).min(self.groups.len() - 1);
                        self.index.select(Some(new_index));
                    }
                },
                Key::Char('\n') => {
                    let selected = self.index.selected().unwrap_or(0);
                    
                    if let Some(group) = self.groups.get(selected) {
                        AlertDialog::new(format!("You selected:\n'{}'", group)).run(ui);
                    }
                },
                Key::Esc | Key::Char('q') => return Some(()),
                _ => {},
            },
            _ => {},
        }

        None
    }

    fn render(&mut self, frame: &mut Frame) {
        let frame_index = format!("Frame {}", self.frame_count);
        let block = Block::default()
            .title(&frame_index)
            .borders(Borders::ALL);

        let items = self.groups.iter().map(|group| Text::raw(group));
        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().modifier(Modifier::ITALIC))
            .highlight_symbol(">>");

        let area = frame.size();
        frame.render_stateful_widget(list, area, &mut self.index);

        self.frame_count += 1;
    }
}

struct AlertDialog {
    text: String,
    offset: u16,
}

impl AlertDialog {
    pub fn new(text: String) -> Self {
        Self {
            text,
            offset: 0,
        }
    }
}

impl View for AlertDialog {
    type Result = ();

    fn update(&mut self, event: ui::Event, _ui: &mut Ui) -> Option<()> {
        match event {
            ui::Event::Termion(Event::Key(key)) =>  match key {
                Key::Esc | Key::Char('q') | Key::Char('\n') => return Some(()),
                Key::Up if self.offset > 0 => self.offset -= 1,
                Key::Down => self.offset += 1,
                _ => {},
            },
            _ => {},
        }

        None
    }

    fn render(&mut self, frame: &mut Frame) {
        let block = Block::default()
            .title("Alert")
            .borders(Borders::ALL);

        let text = [
            Text::raw(&self.text),
        ];
        let content = Paragraph::new(text.iter())
            .block(block)
            .scroll(self.offset)
            .wrap(true);

        let area = frame.size();

        frame.render_widget(content, area);
    }
}
