use std::io;
use std::process::Command;

mod api;

use crossterm::{
    ExecutableCommand,
    terminal::{
        EnterAlternateScreen,
        enable_raw_mode,
        disable_raw_mode,
        LeaveAlternateScreen,
    },
    event:: {
        self,
        Event,
        KeyEventKind,
        KeyCode,
    },
};

use ratatui::{
    Terminal,
    prelude::{
        /*Text,*/ Rect, Widget, Buffer, Layout,
        Color, Style, Backend, Constraint, Line,
        CrosstermBackend, StatefulWidget
    },
    // layout::Flex,
    style::Stylize,
    widgets::{
        Tabs, Table, // Borders,
        Row, Block, Paragraph, TableState
        // List,
    },
};

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    Insert,
}

#[derive(Clone, Copy, PartialEq)]
enum AppTab {
    Search,
    Subs
}

struct App {
    // message: String,
    search_input: String,
    search_input_index: usize,
    running: bool,
    show_info: bool,
    mode: Mode,
    selected_tab: AppTab,
    selected_search_row: TableState,
    search_results: Vec<Vec<String>>,
    tabs: Vec<String>,
}

fn main() -> io::Result<()> {
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    App::new().run(terminal)?;

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

impl App {
    fn new() -> Self {
        Self {
            search_input: String::new(),
            search_input_index: 0,
            running: true,
            show_info: false,
            mode: Mode::Normal,
            selected_tab: AppTab::Search,
            selected_search_row: TableState::default().with_selected(Some(0)),
            search_results: Vec::new(),
            tabs: vec![
                String::from(" Search "),
                String::from(" Subs ")
            ],
        }
    }

    fn run(&mut self,mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        while self.running {
            self.draw(&mut terminal)?;
            self.handle_event()?;
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|frame| {
            match self.mode {
                Mode::Normal => {},
                Mode::Insert => frame.set_cursor(self.search_input_index as u16 + 1, 2),
            }
            frame.render_widget(self, frame.size());
        })?;
        Ok(())
    }

    fn handle_event(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            match self.mode {
                Mode::Normal => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                            KeyCode::Char('j') | KeyCode::Down => {
                                match self.selected_tab {
                                    AppTab::Search => self.select_next_search_row(),
                                    _ => {}
                                }
                            },
                            KeyCode::Char('k') | KeyCode::Up => {
                                match self.selected_tab {
                                    AppTab::Search => self.select_prev_search_row(),
                                    _ => {}
                                }
                            },
                            KeyCode::Char('s') => self.mode = Mode::Insert,
                            KeyCode::Char('i') => self.show_search_info(),
                            KeyCode::Char('1') => self.selected_tab = AppTab::Search,
                            KeyCode::Char('2') => self.selected_tab = AppTab::Subs,
                            KeyCode::Enter     => {
                                match self.selected_search_row.selected() {
                                    Some(i) => {
                                        match self.search_results[i].last() {
                                            Some(s) => {
                                                let _ = Command::new("mpv").args(["--fs", s]).output();
                                            },
                                            None => {},
                                        }
                                    },
                                    None => {},
                                };
                            },
                            _ => {}
                        }
                    }
                },
                Mode::Insert => {
                    match key.code {
                        KeyCode::Char(c)   => self.insert_search_char(c),
                        KeyCode::Backspace => self.delete_search_char(),
                        KeyCode::Enter     => self.submit_input(),
                        KeyCode::Left      => self.move_cursor_left(),
                        KeyCode::Right     => self.move_cursor_right(),
                        KeyCode::Esc       => self.mode = Mode::Normal,
                        _ => {},
                    }
                },
            }
        }
        Ok(())
    }

    fn insert_search_char(&mut self, c: char) {
        self.search_input.insert(self.search_input_index, c);
        self.search_input_index += 1;
    }

    // I don't like how this looks...
    fn delete_search_char(&mut self) {
        if self.search_input_index != 0 {
            let current_index = self.search_input_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.search_input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.search_input.chars().skip(current_index);

            self.search_input = before_char_to_delete.chain(after_char_to_delete).collect();

            self.search_input_index = self.search_input_index
                .saturating_sub(1)
                .clamp(0, self.search_input.chars().count());
        }
    }

    fn move_cursor_left(&mut self) {
        self.search_input_index = self.search_input_index
            .saturating_sub(1)
            .clamp(0, self.search_input.chars().count());
    }

    fn move_cursor_right(&mut self) {
        self.search_input_index = self.search_input_index
            .saturating_add(1)
            .clamp(0, self.search_input.chars().count());
    }

    fn submit_input(&mut self) {
        self.search_results = api::search(&self.search_input);

        self.search_input.clear();
        self.search_input_index = 0;
        self.mode = Mode::Normal;
        self.selected_search_row.select(Some(0));
    }

    fn select_next_search_row(&mut self) {
        let i = match self.selected_search_row.selected() {
            Some(i) => if i >= self.search_results.len() - 1 {i} else {i + 1},
            None => 0
        };

        self.selected_search_row.select(Some(i));
    }

    fn select_prev_search_row(&mut self) {
        let i = match self.selected_search_row.selected() {
            Some(i) => if i == 0 {0} else {i - 1},
            None => 0
        };

        self.selected_search_row.select(Some(i));
    }

    fn show_search_info(&mut self) {
        if !self.search_results.is_empty() {
            self.show_info = !self.show_info;
        }
    }

    fn render_tabs(&mut self, area: Rect, buffer: &mut Buffer) {
        Tabs::new(self.tabs.iter().map(|i| i.to_string()))
            .select(self.selected_tab as usize)
            .style(Style::default().bg(Color::Black))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black).bold())
            .divider("")
            .padding("", " ")
            .render(area, buffer);
    }

    fn render_search(&mut self, area: Rect, buffer: &mut Buffer) {
        let search_layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
        ]);

        let [input, results] = search_layout.areas(area);

        let input_style = match self.mode {
            Mode::Insert => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };

        Paragraph::new(self.search_input.as_str())
            .block(Block::bordered().title("Input").style(input_style))
            .render(input, buffer);

        let result_layout = Layout::horizontal([
            Constraint::Percentage(if self.show_info { 70 } else { 100 }),
            Constraint::Min(0),
        ]);

        let [results_list, info] = result_layout.areas(results);

        let results_block = Block::bordered()
            .title("Results");

        let table_data: Vec<_> = self.search_results
            .iter()
            .map(|data| Row::new(data.iter().map(|cell| cell.clone())))
            .collect();

        if table_data.is_empty() {
            Paragraph::new("No results, yet...").block(results_block).render(results, buffer);
        } else {
            StatefulWidget::render(
                Table::new(
                    table_data,
                    [Constraint::Percentage(50), Constraint::Percentage(30), Constraint::Percentage(20)]
                )
                .header(
                    Row::new(vec!["Title", "Channel", "Duration"])
                    .style(Style::default().bold().fg(Color::Green))
                    .bottom_margin(0)
                )
                .highlight_style(Style::new().reversed())
                .block(results_block),
                results_list, buffer, &mut self.selected_search_row
            );

            let info_block = Block::bordered()
                .title("Info");

            if self.show_info {
                let mut index = 0;
                if let Some(i) = self.selected_search_row.selected() {
                    index = i;
                };

                let lines = vec![
                    Line::from("Title: ").style(Color::Green),
                    Line::from(self.search_results[index][0].clone()),
                    Line::from("Channel: ").style(Color::Green),
                    Line::from(self.search_results[index][1].clone()),
                    Line::from("Duration: ").style(Color::Green),
                    Line::from(self.search_results[index][2].clone()),
                    Line::from("Link: ").style(Color::Green),
                    Line::from(format!("https://youtube.com/watch?v={}", self.search_results[index][3].clone())),
                ];

                Paragraph::new(lines)
                    .block(info_block)
                    .render(info, buffer);
            }
        }

        // List::new(vec!["Item", "Item", "Item", "Item", "Item", "Item", "Item"])
        //     .block(results_block)
        //     .render(results, buffer);
    }

    fn render_subs(&mut self, area: Rect, buffer: &mut Buffer) {
        let subs_layout = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ]);

        let [channels, feed] = subs_layout.areas(area);

        Block::bordered()
            .title("Channels")
            .render(channels, buffer);

        Block::bordered()
            .title("Feed")
            .render(feed, buffer);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let main_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
        ]);

        let [header, content] = main_layout.areas(area);

        self.render_tabs(header, buffer);

        match self.selected_tab {
            AppTab::Search => self.render_search(content, buffer),
            AppTab::Subs => self.render_subs(content, buffer),
        }
    }
}
