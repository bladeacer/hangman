use color_eyre::Result;
use random_word::Lang;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};
use tui_input::{Input, backend::crossterm::EventHandler};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

struct App {
    rand_str: String,
    replaced_str: String,
    replaced_sym: char,
    is_show_rules: bool,
    is_show_main_menu: bool,
    input: Input,
    input_mode: InputMode,
    incorrect_guesses: usize,
    max_incorrect_guesses: usize,
    is_game_over: bool,
    is_winner: bool,
    debug_mode: bool, // Whether debug mode is enabled
}

impl App {
    fn new() -> Self {
        let rand_str = random_word::get(Lang::En).to_owned();
        let replaced_sym: char = '_';
        let replaced_str: String = replace_non_vowels(&rand_str, replaced_sym);
        let is_show_rules: bool = false;
        let is_show_main_menu: bool = true;
        let input = Input::default();
        let input_mode = InputMode::Normal;
        let incorrect_guesses = 0;
        let max_incorrect_guesses = 7;
        let is_game_over = false;
        let is_winner = false;
        let debug_mode = false; // Debug mode is off by default

        Self {
            rand_str,
            replaced_str,
            replaced_sym,
            is_show_rules,
            is_show_main_menu,
            input,
            input_mode,
            incorrect_guesses,
            max_incorrect_guesses,
            is_game_over,
            is_winner,
            debug_mode,
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let event = event::read()?;
            if let Event::Key(key) = event {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('n') => {
                            self.is_show_main_menu = false;
                            self.reset_game();
                        }
                        KeyCode::Char('m') => {
                            self.is_show_rules = false;
                            self.is_show_main_menu = true; 
                        }
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    }, 
                    InputMode::Editing => {}
                }
                if self.is_show_main_menu {
                    match key.code {
                        KeyCode::Char('r') => {
                            self.is_show_main_menu = false;
                            self.is_show_rules = true;
                        }
                        _ => {}
                    }
                } else if self.is_show_rules {
                    match key.code {
                        KeyCode::Char('n') => {
                            self.is_show_rules = false;
                        },
                        _ => {}
                    }
                }
                if !self.is_game_over && !self.is_winner {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('d') => self.debug_mode = !self.debug_mode, // Toggle debug mode
                            KeyCode::Char('e') => self.input_mode = InputMode::Editing,
                            KeyCode::Char('r') => self.is_show_rules = true, // Show rules
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => self.process_guess(),
                            KeyCode::Esc => self.input_mode = InputMode::Normal,
                            _ => {
                                // Allow editing (e.g., backspace) while enforcing the character limit for new input
                                if self.input.value().len() < self.rand_str.len() || matches!(key.code, KeyCode::Backspace) {
                                    self.input.handle_event(&event);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        if self.is_show_main_menu {
            // Use 100% of the height for the main menu
            let menu_text = Paragraph::new(
                r#"
Welcome to Hangman!

Credits:
- Developer: bladeacer
- Libraries used: ratatui, random, crossterm, tui_input

Controls:
- Press 'n' to start a new game
- Press 'r' to view the rules
- Press 'q' to quit"#)
            .wrap(ratatui::widgets::Wrap { trim: true }) // Enable text wrapping
            .block(Block::bordered().title("Main Menu"));
            frame.render_widget(menu_text, frame.area());
        } else if self.is_show_rules {
            // Use 100% of the height for the rules page
            let rules_text = Paragraph::new(
                r#"
1. Guess the word by entering letters.
2. You can also guess the full word.
3. You lose if you exceed the maximum incorrect guesses.

Controls:
- Press 'm' to return to the main menu
- Press 'n' to start a new game.
- Press 'q' to quit.
"#)
            .wrap(ratatui::widgets::Wrap { trim: true }) // Enable text wrapping
            .block(Block::bordered().title("Rules"));
            frame.render_widget(rules_text, frame.area());
        } else {
            // Layout for the game state
            let [top, input_area, controls_area] = Layout::vertical([
                Constraint::Percentage(50), // Allocate 50% of the height for the top section
                Constraint::Length(3),      // Fixed height for the input area
                Constraint::Percentage(40), // Allocate 40% of the height for the controls area
            ])
            .areas(frame.area());

            // Display the game state, including the incorrect guesses
            let displayed_word = Paragraph::new(format!(
                "Word: {}\nGuesses: {}/{}",
                self.replaced_str, self.incorrect_guesses, self.max_incorrect_guesses
            ))
            .wrap(ratatui::widgets::Wrap { trim: true }) // Enable text wrapping
            .block(Block::bordered().title("Hangman"));
            frame.render_widget(displayed_word, top);

            let width = input_area.width.max(3) - 3;
            let scroll = self.input.visual_scroll(width as usize);
            let style = match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Color::Yellow.into(),
            };
            let input = Paragraph::new(self.input.value())
                .style(style)
                .scroll((0, scroll as u16))
                .block(Block::bordered().title("Input"));
            frame.render_widget(input, input_area);

            if self.input_mode == InputMode::Editing {
                let x = self.input.visual_cursor().max(scroll) - scroll + 1;
                frame.set_cursor_position((input_area.x + x as u16, input_area.y + 1));
            }

            // Split the controls area into two sections: controls and debug info
            let sections = Layout::horizontal([
                Constraint::Percentage(50), 
                Constraint::Percentage(50), 
            ])
            .split(controls_area);

            let controls_section = sections[0];
            let debug_section = sections[1];

            // Display controls based on the input mode
            let controls_text = if self.is_game_over && self.is_winner {
                "You win! Press 'n' to start a new game, 'm' to return to the main menu, or 'q' to quit.".to_string()
            } else if self.is_game_over {
                "You lose! Press 'n' to start a new game, 'm' to return to the main menu, or 'q' to quit.".to_string()
            } else if self.input_mode == InputMode::Editing {
                "Press Esc to escape.".to_string()
            } else {
                "Press 'e' to edit, 'r' to view rules, 'm' to return to main menu, 'n' to start a new game, or 'q' to quit.".to_string()
            };
            let controls = Paragraph::new(controls_text)
                .wrap(ratatui::widgets::Wrap { trim: true }) // Enable text wrapping
                .block(Block::bordered().title("Controls"));
            frame.render_widget(controls, controls_section);

            // Display debug info if debug mode is enabled
            let debug_text = if self.debug_mode {
                format!("Debug Mode: Base word is '{}'", self.rand_str)
            } else {
                "Debug Mode: Disabled".to_string()
            };
            let debug_paragraph = Paragraph::new(debug_text)
                .style(Style::default().fg(Color::Red))
                .block(Block::bordered().title("Debug Info"));
            frame.render_widget(debug_paragraph, debug_section);
        }
    }

    fn process_guess(&mut self) {
        let guess = self.input.value_and_reset().to_lowercase();

        if guess.len() == 1 {
            // Single character guess
            let guessed_char = guess.chars().next().unwrap();
            if self.rand_str.contains(guessed_char) {
                // Correct character guess: reveal the character
                self.replaced_str = self
                    .rand_str
                    .chars()
                    .map(|ch| if ch == guessed_char || self.replaced_str.contains(ch) { ch } else { self.replaced_sym })
                    .collect();

                // Check if the user has won
                if self.replaced_str == self.rand_str {
                    self.is_game_over = true;
                    self.is_winner = true;
                }
            } else {
                // Incorrect character guess
                self.incorrect_guesses += 1;

                // Check if the user has lost
                if self.incorrect_guesses >= self.max_incorrect_guesses {
                    self.is_game_over = true;
                    self.is_winner = false;
                }
            }
        } else if guess == self.rand_str {
            // Full word guess
            self.is_game_over = true;
            self.is_winner = true;
            self.replaced_str = self.rand_str.clone();
        } else {
            // Incorrect full word guess
            let mut all_incorrect = true;

            // Check each character in the guessed word
            for ch in guess.chars() {
                if self.rand_str.contains(ch) {
                    // Reveal correct characters
                    self.replaced_str = self
                        .rand_str
                        .chars()
                        .map(|base_ch| if base_ch == ch || self.replaced_str.contains(base_ch) { base_ch } else { self.replaced_sym })
                        .collect();
                    all_incorrect = false; // At least one character is correct
                }
            }

            // Increment incorrect guesses only if all characters are incorrect
            if all_incorrect {
                self.incorrect_guesses += 1;
            }

            // Check if the user has won after revealing correct characters
            if self.replaced_str == self.rand_str {
                self.is_game_over = true;
                self.is_winner = true;
            }

            // Check if the user has lost
            if self.incorrect_guesses >= self.max_incorrect_guesses {
                self.is_game_over = true;
                self.is_winner = false;
            }
        }
    }

    fn reset_game(&mut self) {
        self.rand_str = random_word::get(Lang::En).to_owned();
        self.replaced_str = replace_non_vowels(&self.rand_str, self.replaced_sym);
        self.incorrect_guesses = 0;
        self.is_game_over = false;
        self.is_winner = false;
        self.input = Input::default(); // Clear the input
    }
}

fn replace_non_vowels(original: &str, replacement_symbol: char) -> String {
    original
        .chars()
        .map(|c| if "aeiouAEIOU".contains(c) { c } else { replacement_symbol })
        .collect()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,  
    Editing, 
}