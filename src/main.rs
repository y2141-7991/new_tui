use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::widgets::ListState;
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use ratatui::{
    prelude::*,
    style::{Color, palette::tailwind},
    text::Line,
    widgets::{block::Title, *},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
const DEFAULT_STYLE: Style = Style::new().fg(Color::Green).bg(Color::Black);

#[derive(Debug, Clone)]
pub struct ListStateWrapper {
    max: usize,
    state: ListState,
}

const GAUGE1_COLOR: Color = tailwind::RED.c800;

impl ListStateWrapper {
    pub fn new(max: usize) -> Self {
        Self {
            max,
            state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn increment(&mut self) {
        let Some(existing) = self.state.selected() else {
            self.select(0);
            return;
        };
        let next = (existing + 1).min(self.max);
        self.select(next);
    }

    pub fn decrement(&mut self) {
        let Some(existing) = self.state.selected() else {
            self.state.select(Some(self.max));
            return;
        };
        let next = existing.saturating_sub(1);
        self.select(next);
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn select(&mut self, new_idx: usize) {
        let new_idx = new_idx.min(self.max);
        self.state.select(Some(new_idx));
    }
}

impl AsMut<ListState> for ListStateWrapper {
    fn as_mut(&mut self) -> &mut ListState {
        &mut self.state
    }
}

impl AsRef<ListState> for ListStateWrapper {
    fn as_ref(&self) -> &ListState {
        &self.state
    }
}

enum AppEvent {
    MoveUp,
    MoveDown,
    MoveRight,
    MoveLeft,
}

#[derive(Debug, Clone)]
struct Button<'a> {
    label: Line<'a>,
    theme: Theme,
    button_state: ButtonState,
}

#[derive(Debug, Clone, Copy)]
struct Theme {
    text: Color,
    background: Color,
    highlight: Color,
    shadow: Color,
}

#[derive(Debug, Clone, Copy)]
enum ButtonState {
    Normal,
    Selected,
    Active,
}

const GREEN: Theme = Theme {
    text: Color::DarkGray,
    background: Color::LightGreen,
    highlight: Color::Green,
    shadow: Color::DarkGray,
};

impl<'a> Button<'a> {
    fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Button {
            label: label.into(),
            theme: GREEN,
            button_state: ButtonState::Normal,
        }
    }

    const fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    const fn button_state(mut self, button_state: ButtonState) -> Self {
        self.button_state = button_state;
        self
    }
}

impl Button<'_> {
    const fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.button_state {
            ButtonState::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            ButtonState::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            ButtonState::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
        }
    }
}

impl<'a> Widget for Button<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let background
    }
}

#[derive(Debug, Clone)]
pub struct App {
    main_menu_state: ListStateWrapper,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let horizontal_ =
            Layout::horizontal([Constraint::Percentage(10), Constraint::Percentage(90)]);
        let [menu_area, display_area] = horizontal_.areas(area);

        let vertical_ = Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(10), Constraint::Percentage(10)]);

        let [draw, progress_bar, map] = vertical_.areas(display_area);

        let mut constraints = vec![];
        constraints.extend(std::iter::repeat(Constraint::Percentage(17)).take(5));
        constraints.extend(std::iter::repeat(Constraint::Percentage(5)).take(3));

        let hr = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints);

        let [p1, p2, p3, p4, p5, p6, p7, p8] = hr.areas(map);

        self.render_audio_folder(menu_area, buf);

        self.render_page(draw, buf);

        self.render_progress_bar(progress_bar, buf);

        self.render_button("Recovery ⟲".to_string(), p1, buf);
        self.render_button("Backward 5s↩".to_string(), p2, buf);
        self.render_button("Play ▶".to_string(), p3, buf);
        self.render_button("Pause ❙❙".to_string(), p4, buf);
        self.render_button("Forward ↪5s".to_string(), p5, buf);
        self.render_button("▶▶".to_string(), p6, buf);
        self.render_button("🕪".to_string(), p7, buf);
        self.render_button("🕩".to_string(), p8, buf);
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            main_menu_state: ListStateWrapper::new(4),
        }
    }
    pub fn tick(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), Box<dyn Error>> {
        self.draw(terminal)?;
        Ok(())
    }
    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.area()))?;
        Ok(())
    }
    fn event(&mut self, event: AppEvent) {
        match event {
            AppEvent::MoveUp => println!("MoveUp"),
            AppEvent::MoveDown => println!("MoveDown"),
            AppEvent::MoveLeft => println!("MoveLeft"),
            AppEvent::MoveRight => println!("MoveRight"),
            _ => println!("")
        }
    }


    fn render_audio_folder(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Audio Folder")
            .title_alignment(Alignment::Center)
            .border_set(ratatui::symbols::border::PLAIN)
            // .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .style(DEFAULT_STYLE);
        let content_area = block.inner(area);
        block.render(area, buf);
        let list = List::new([
            ListItem::from("Home"),
            "Work".into(),
            "Open Source".into(),
            "Education".into(),
        ]);
        let list = list.highlight_style(Style::new().bg(Color::Green).fg(Color::Black));
        StatefulWidget::render(list, content_area, buf, self.main_menu_state.as_mut());
    }
    fn render_progress_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().style(DEFAULT_STYLE);
        // let inner_area = block.inner(area);
        let g = Gauge::default().block(block).gauge_style(GAUGE1_COLOR);
        g.render(area, buf);
    }
    fn render_button(&mut self, text: String, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().style(DEFAULT_STYLE);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(1),
                Constraint::Percentage(40),
            ])
            .split(inner_area);

        let p = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(DEFAULT_STYLE);
        p.render(vertical_chunks[1], buf);
    }
    fn render_page(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("▶ ❙❙  ↪5s  5s↩   ⟲ ")
            .title_alignment(Alignment::Center)
            .border_set(ratatui::symbols::border::PLAIN)
            // .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .style(DEFAULT_STYLE);
        let content_area = block.inner(area);
        block.render(area, buf);
        let list = List::new([
            ListItem::from("Home"),
            "Work".into(),
            "Open Source".into(),
            "Education".into(),
        ]);
        let list = list.highlight_style(Style::new().bg(Color::Green).fg(Color::Black));
        StatefulWidget::render(list, content_area, buf, self.main_menu_state.as_mut());
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        app.tick(&mut terminal)?;

        // Input handling
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
