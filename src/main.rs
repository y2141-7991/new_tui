use std::{error::Error, fs::File, io, time::Duration};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use glob::glob;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, palette::tailwind},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use ratatui::{
    symbols::block,
    widgets::{Gauge, ListState, Padding, Widget},
};
use rodio::{Decoder, OutputStream, Sink, Source};

const CUSTOM_LABEL_COLOR: Color = tailwind::CYAN.c800;
const GAUGE3_COLOR: Color = tailwind::BLUE.c800;

struct Buttons<'a> {
    states: Vec<&'a str>,
    
}

enum ButtonStates {
    
}

struct AudioService {
    _stream: OutputStream,
    sink: Sink,
    audio_event: AudioEvent,
    speed: f32,
    length: usize,
}

#[derive(Debug, Default, Clone)]
enum AudioEvent {
    #[default]
    Play,
    Pause,
    SpeedUp,
    SpeedDown,
    SeekForward,
    SeekBackward,
}

impl AudioService {
    fn new() -> Self {
        let (_stream, _hanlder) = OutputStream::try_default().expect("Can not init OutputStream");
        let sink = Sink::try_new(&_hanlder).expect("Can not init Sink and PlayError");
        Self {
            _stream,
            sink,
            audio_event: AudioEvent::default(),
            speed: 1.0,
            length: 0,
        }
    }
    fn play(&mut self, f: String) {
        let file = File::open(f).expect("Can not file this file");
        let source = Decoder::new(file).expect("Decoder Error");
        self.length = if let Some(d) = source.total_duration() {
            d.as_secs() as usize
        } else {
            0
        };
        self.sink.append(source);
        self.sink.play();
    }
    fn pause(&mut self) {
        self.sink.pause();
    }
    fn speed_up(&mut self) {
        self.speed += 0.25;
        self.sink.set_speed(self.speed);
    }
    fn speed_down(&mut self) {
        self.speed -= 0.25;
        self.sink.set_speed(self.speed);
    }
    fn seek_forward(&mut self) {
        let mut current = self.sink.get_pos();
        if self.length > 5 && (current.as_secs() as usize) >= (self.length - 5) {
            current = Duration::from_secs(self.length as u64)
        } else {
            current += Duration::from_secs(5);
        }
        self.sink.try_seek(current).expect("Can not seek more");
    }
    fn seek_backward(&mut self) {
        let mut current = self.sink.get_pos();
        if current.as_secs() < 5 {
            current = Duration::from_secs(0)
        } else {
            current -= Duration::from_secs(5);
        }
        self.sink.try_seek(current).expect("Can not seek more");
    }
    fn get_current_position(&self) -> Duration {
        self.sink.get_pos()
    }
}

#[derive(Debug)]
struct AudioFolder<'a> {
    path: &'a str,
    files: Vec<String>,
}

impl AudioFolder<'_> {
    fn new(path: &'static str) -> Self {
        Self {
            path: path,
            files: Vec::new(),
        }
    }
    fn load_mp3_file(&mut self) {
        let path = match glob(&self.path) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Invalid file path {}", &self.path);
                return;
            }
        };
        let mut files: Vec<_> = Vec::new();
        for entry in path {
            match entry {
                Ok(file) => {
                    let f = file.display().to_string();
                    files.push(f);
                }
                Err(e) => {
                    eprintln!("Glob error {}", e);
                    return;
                }
            };
        }
        self.files = files;
    }
}

#[derive(PartialEq)]
enum Focus {
    FolderList,
    Buttons,
}

struct App<'a> {
    folder_state: ListState,

    audio_service: AudioService,
    audio_folder: AudioFolder<'a>,
    buttons: Vec<&'a str>,
    button_index: usize,
    focus: Focus,
    tick_rate: Duration,
    should_quit: bool,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let path = "sample_mp3/*";
        let mut audio_folder = AudioFolder::new(path);
        audio_folder.load_mp3_file();

        let mut folder_state = ListState::default();
        folder_state.select(Some(0));
        Self {
            folder_state,

            audio_service: AudioService::new(),
            audio_folder: audio_folder,
            buttons: vec!["-5s↩", "+↪5s", "⏴⏴", "▶", "⏸", "⏵⏵"],
            button_index: 0,
            focus: Focus::FolderList,
            tick_rate: Duration::from_millis(200),
            should_quit: false,
        }
    }

    fn next_folder(&mut self) {
        let i = match self.folder_state.selected() {
            Some(i) => (i + 1) % self.audio_folder.files.len(),
            None => 0,
        };
        self.folder_state.select(Some(i));
    }

    fn prev_folder(&mut self) {
        let i = match self.folder_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.audio_folder.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.folder_state.select(Some(i));
    }

    fn next_button(&mut self) {
        self.button_index = (self.button_index + 1) % self.buttons.len();
    }

    fn prev_button(&mut self) {
        self.button_index = if self.button_index == 0 {
            self.buttons.len() - 1
        } else {
            self.button_index - 1
        };
    }
}

impl App<'_> {
    fn handle_event(&mut self) -> Result<(), std::io::Error> {
        if event::poll(self.tick_rate)? {
            if let CEvent::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Tab => {
                        self.focus = if self.focus == Focus::Buttons {
                            Focus::FolderList
                        } else {
                            Focus::Buttons
                        }
                    }

                    KeyCode::Char('j') | KeyCode::Down => {
                        if self.focus == Focus::FolderList {
                            self.next_folder();
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if self.focus == Focus::FolderList {
                            self.prev_folder();
                        }
                    }

                    KeyCode::Char('h') | KeyCode::Left => {
                        if self.focus == Focus::Buttons {
                            self.prev_button();
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if self.focus == Focus::Buttons {
                            self.next_button();
                        }
                    }
                    KeyCode::Char(' ') => {
                        if self.focus == Focus::Buttons {
                            if let Some(i) = self.folder_state.selected() {
                                match self.buttons[self.button_index] {
                                    "▶" => {
                                        self.audio_service.play(self.audio_folder.files[i].clone())
                                    }
                                    "❚❚" => self.audio_service.pause(),
                                    "▶▶" => self.audio_service.speed_up(),
                                    "◀◀" => self.audio_service.speed_down(),
                                    "+↪5s" => self.audio_service.seek_forward(),
                                    "-5s↩" => self.audio_service.seek_backward(),
                                    _ => println!(""),
                                }
                            }
                        }
                    }
                    _ => eprintln!("Key is not handled {:?}", key_event),
                }
            }
        }

        Ok(())
    }
}

impl App<'_> {
    fn render_main_page(&mut self, frame: &mut ratatui::Frame) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(frame.area());
        self.render_list_files(frame, horizontal[0]);

        let vertical = Layout::vertical([
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(horizontal[1]);

        self.render_progress_bar(frame, vertical[1]);
        self.render_button(frame, vertical[2]);
    }

    fn render_list_files(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let folder_items: Vec<_> = self
            .audio_folder
            .files
            .iter()
            .map(|f| ListItem::new(f.clone()))
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Queue")
            .title_alignment(ratatui::layout::Alignment::Center);
        let hs = Style::default().fg(Color::Black).bg(Color::Green);

        let folder_list = List::new(folder_items)
            .block(block)
            .highlight_style(hs)
            .highlight_symbol(" >");
        frame.render_stateful_widget(folder_list, area, &mut self.folder_state);
    }

    fn render_button(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let button_chunks = Layout::horizontal([Constraint::Percentage(20); 6]).split(area);

        for (i, button) in self.buttons.iter().enumerate() {
            let is_selected = self.focus == Focus::Buttons && self.button_index == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1));
            let inner = block.inner(button_chunks[i]);
            let vertical = Layout::vertical([
                Constraint::Percentage(40),
                Constraint::Length(1),
                Constraint::Percentage(40),
            ])
            .split(inner);

            let p = Paragraph::new(*button)
                .style(style)
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(p, vertical[1]);
            frame.render_widget(block, button_chunks[i]);
        }
    }
    fn render_progress_bar(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let elapsed_time = formart_duration(self.audio_service.get_current_position());
        let total = formart_duration(Duration::new(self.audio_service.length as u64, 0));
        let ratio = if self.audio_service.length == 0 {
            0.0
        } else {
            self.audio_service.get_current_position().as_secs_f64()
                / self.audio_service.length as f64
        };

        let span = Span::styled(
            format!("{}/{}", elapsed_time, total),
            Style::new().fg(CUSTOM_LABEL_COLOR),
        );
        let gauge = Gauge::default()
            .block(Block::default().title("Time").borders(Borders::ALL))
            .gauge_style(GAUGE3_COLOR)
            .ratio(ratio)
            .label(span);
        frame.render_widget(gauge, area);
    }
}

fn formart_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    while !app.should_quit {
        terminal.draw(|f| {
            app.render_main_page(f);
        })?;

        app.handle_event()?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
