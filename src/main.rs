use std::io;

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
        Color, Style, Backend, Constraint,
        CrosstermBackend, StatefulWidget
    },
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
            mode: Mode::Normal,
            selected_tab: AppTab::Search,
            selected_search_row: TableState::default().with_selected(Some(0)),
            search_results: vec![
                vec![
                    String::from("3D in TypeScript using Ray Casting"),
                    String::from("Tsoding Daily"),
                    String::from("video"),
                ],
                vec![
                    String::from("Unreasonable Effectiveness of Abstractions"),
                    String::from("Tsoding Daily"),
                    String::from("video")
                ],
                vec![
                    String::from("Is John Carmack Right about UI?!"),
                    String::from("Tsoding Daily"),
                    String::from("video")
                ],
                vec![
                    String::from("Can C actually do Perfect Bézier Curves?"),
                    String::from("Tsoding Daily"),
                    String::from("video")
                ],
            ],
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
                            KeyCode::Char('i') => self.mode = Mode::Insert,
                            KeyCode::Char('1') => self.selected_tab = AppTab::Search,
                            KeyCode::Char('2') => self.selected_tab = AppTab::Subs,
                            _ => {}
                        }
                    }
                },
                Mode::Insert => {
                    match key.code {
                        KeyCode::Char(c)   => self.insert_search_char(c),
                        KeyCode::Backspace => self.delete_search_char(),
                        KeyCode::Enter     => self.reset_input(),
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

    fn reset_input(&mut self) {
        self.search_input.clear();
        self.search_input_index = 0;
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

        // Terminal::set_cursor(input.x, input.y);

        let input_style = match self.mode {
            Mode::Insert => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };

        Paragraph::new(self.search_input.as_str())
            .block(Block::bordered().title("Input").style(input_style))
            .render(input, buffer);

        let results_block = Block::bordered()
            .title("Results");

        let table_data: Vec<_> = self.search_results
            .iter()
            .map(|data| Row::new(data.iter().map(|cell| cell.clone())))
            .collect();

        StatefulWidget::render(
            Table::new(
                table_data,
                [Constraint::Percentage(50), Constraint::Percentage(30), Constraint::Percentage(20)]
            )
            .header(
                Row::new(vec!["Title", "Channel", "Type"])
                .style(Style::default().bold().fg(Color::Green))
                .bottom_margin(0)
            )
            // .highlight_style(Style::new().bg(Color::Blue).fg(Color::White))
            .highlight_style(Style::new().reversed())
            .block(results_block),
            results, buffer, &mut self.selected_search_row
        );

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
        // Text::raw("Subs Tab").render(area, buffer);
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
