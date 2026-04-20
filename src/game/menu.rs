// src/game/menu.rs
use winit::keyboard::KeyCode;

pub enum MenuState { Main, LoadGame }

pub enum EngineAction { 
    Continue, 
    Quit,
    LoadWorld(&'static str),
    ReturnToMenu, // FIXED: Added missing variant for when the user hits Escape!
}

pub struct MenuSystem {
    pub state: MenuState,
    pub cursor_index: usize,
}

impl MenuSystem {
    pub fn new() -> Self { Self { state: MenuState::Main, cursor_index: 0 } }

    pub fn get_current_options(&self) -> Vec<&'static str> {
        match self.state {
            MenuState::Main => vec!["LOAD GAME", "SETTINGS", "QUIT"],
            MenuState::LoadGame => vec!["WORLD01", "WORLD02", "BACK"],
        }
    }

    pub fn handle_input(&mut self, keycode: KeyCode) -> EngineAction {
        let count = self.get_current_options().len();
        match keycode {
            KeyCode::ArrowUp => self.cursor_index = if self.cursor_index > 0 { self.cursor_index - 1 } else { count - 1 },
            KeyCode::ArrowDown => self.cursor_index = if self.cursor_index < count - 1 { self.cursor_index + 1 } else { 0 },
            KeyCode::Enter => return self.select_current_option(),
            _ => {}
        }
        EngineAction::Continue
    }

    fn select_current_option(&mut self) -> EngineAction {
        match self.state {
            MenuState::Main => match self.cursor_index {
                0 => { self.state = MenuState::LoadGame; self.cursor_index = 0; }
                1 => tracing::info!("Settings not implemented!"),
                2 => return EngineAction::Quit,
                _ => {}
            },
            MenuState::LoadGame => match self.cursor_index {
                0 => return EngineAction::LoadWorld("WORLD01"),
                1 => return EngineAction::LoadWorld("WORLD02"),
                2 => { self.state = MenuState::Main; self.cursor_index = 0; }
                _ => {}
            },
        }
        EngineAction::Continue
    }
}