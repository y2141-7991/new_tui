use std::time::Duration;

use ratatui::widgets::ListState;

use crate::{AudioFolder, AudioService, Focus};




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