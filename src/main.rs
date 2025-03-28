use std::time::{Duration, Instant};
use rand::Rng;
use rand::seq::SliceRandom;
use color_eyre::Result;
use random_word::Lang;
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Layout, Rect};
// use ratatui::style::{Color, Modifier, Style, Stylize};
// use ratatui::symbols::{self, Marker};
// use ratatui::text::{Line, Span};
// use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, LegendPosition, Paragraph};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

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
    replaced_sym: char
}

impl App {
    fn new() -> Self {
        let rand_str = random_word::get(Lang::En).to_owned();
        let replaced_sym: char = '_';
        let replaced_str: String = replace_non_vowels(&rand_str, replaced_sym);
        Self {
            rand_str,
            replaced_str,
            replaced_sym
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        return Ok(());
                    }
                    if key.code == KeyCode::Char('r') {
                        let temp_str = random_word::get(Lang::En);
                        self.rand_str = temp_str.to_owned();
                        self.replaced_str = replace_non_vowels(&temp_str, self.replaced_sym);

                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let [top, _bottom] = Layout::vertical([Constraint::Fill(1); 2]).areas(frame.area());

        let [greeting_rect, controls_rect] = Layout::horizontal([Constraint::Fill(1), Constraint::Percentage(50)]).areas(top);

        self.render_rand_text(frame, greeting_rect);
        self.render_controls(frame, controls_rect);
    }

    fn render_rand_text(&self, frame: &mut Frame, area: Rect) {
        let rand_word = &self.rand_str;
        let mut greeting_text = String::from("Base word: ");
        greeting_text.push_str(rand_word);
        greeting_text.push_str("\nDisplayed word: ");
        greeting_text.push_str(&self.replaced_str);
        let greeting = Paragraph::new(greeting_text);
        frame.render_widget(greeting, area);
    }

    fn render_controls (&self, frame: &mut Frame, area: Rect) {
        let controls_str: String = String::from("Press 'q' to exit\nPress 'r' to load a new word");
        let controls = Paragraph::new(controls_str);
        frame.render_widget(controls, area);
    }


}

fn replace_non_vowels(original: &str, replacement_symbol: char) -> String {
    let vowels: Vec<char> = original.chars().filter(|c| "aeiouAEIOU".contains(*c)).collect();

    if vowels.is_empty() {
        let mut result = String::new();
        for _ in original.chars() {
            result.push(replacement_symbol);
        }
        return result;
    }

    let mut rng = rand::rng();
    let num_vowels_to_replace = rng.random_range(1..=vowels.len().min(3));

    let mut result = String::new();
    let vowel_indices: Vec<usize> = original
        .char_indices()
        .filter(|(_, c)| "aeiouAEIOU".contains(*c))
        .map(|(i, _)| i)
        .collect();

    let mut shuffled_vowel_indices = vowel_indices.clone();
    shuffled_vowel_indices.shuffle(&mut rng);
    let indices_to_replace = &shuffled_vowel_indices[0..num_vowels_to_replace];

    let mut shuffled_vowels = vowels.clone();
    shuffled_vowels.shuffle(&mut rng);

    let mut vowel_index = 0;
    let mut vowel_count = 0;
    for (index, char) in original.char_indices() {
        if "aeiouAEIOU".contains(char) && vowel_count < 3 {
            if indices_to_replace.contains(&index) && vowel_index < shuffled_vowels.len() {
                result.push(shuffled_vowels[vowel_index]);
                vowel_index += 1;
                vowel_count += 1;
            } else {
                result.push(char);
                vowel_count += 1
            }
        } else {
            result.push(replacement_symbol);
        } 
    }

    result
}
