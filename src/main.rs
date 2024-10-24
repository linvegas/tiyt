use std::io;
use std::env;
use tokio;

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
        Row, Block, Paragraph, TableState,
        List,
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

struct SearchItem {
    title: String,
    channel: String,
    duration: String,
    publish_time: String,
    view_count: String,
    link: String,
    description: String,
}

struct SearchResults {
    items: Vec<SearchItem>,
}

struct App {
    search_input: String,
    search_input_index: usize,
    running: bool,
    show_info: bool,
    mode: Mode,
    selected_tab: AppTab,
    selected_search_row: TableState,
    search_results: SearchResults,
    tabs: Vec<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    App::new().run(terminal).await?;

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
            search_results: SearchResults {items: Vec::new()},
            tabs: vec![
                String::from(" Search "),
                String::from(" Subs ")
            ],
        }
    }

    async fn run(&mut self,mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        while self.running {
            self.draw(&mut terminal)?;
            self.handle_event().await?;
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

    async fn handle_event(&mut self) -> io::Result<()> {
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
                            KeyCode::Enter     => self.play_video(),
                            _ => {}
                        }
                    }
                },
                Mode::Insert => {
                    match key.code {
                        KeyCode::Char(c)   => self.insert_search_char(c),
                        KeyCode::Backspace => self.delete_search_char(),
                        KeyCode::Enter     => self.submit_input().await,
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

    async fn submit_input(&mut self) {
        let data: Vec<Vec<String>> = api::get_search_list(&self.search_input).await;

        self.search_results = SearchResults {
            items: data.into_iter().map(|item| {
                SearchItem {
                    title:        item[0].clone(),
                    channel:      item[1].clone(),
                    publish_time: item[2].clone(),
                    duration:     item[3].clone(),
                    view_count:   item[4].clone(),
                    link:         item[5].clone(),
                    description:  item[6].clone(),
                }
            }).collect()
        };

        self.search_input.clear();
        self.search_input_index = 0;
        self.mode = Mode::Normal;
        self.selected_search_row.select(Some(0));
    }

    fn play_video(&mut self) {
        if let Some(index) = self.selected_search_row.selected() {
            let link = &self.search_results.items[index].link;

            let mpv_option = env::var("MPV_OPTION");

            let mpv_option = match mpv_option {
                Ok(s) => s,
                Err(_) => String::new(),
            };

            let _ = std::process::Command::new("mpv")
                .args(mpv_option.split_whitespace())
                .arg(link)
                .stdout(std::process::Stdio::null())
                .spawn();
        }
    }

    fn select_next_search_row(&mut self) {
        let i = match self.selected_search_row.selected() {
            Some(i) => if i >= self.search_results.items.len() - 1 {i} else {i + 1},
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
        if !self.search_results.items.is_empty() {
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

        let result_layout = Layout::vertical([
            Constraint::Percentage(if self.show_info { 30 } else { 100 }),
            Constraint::Min(0),
        ]);

        let [results_list, info] = result_layout.areas(results);

        let results_block = Block::bordered()
            .title("Results");

        let table_data: Vec<_> = self.search_results.items
            .iter()
            .map(|r| Row::new(vec![r.title.clone(), r.channel.clone(), r.duration.clone()]))
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
                    Line::from(self.search_results.items[index].title.clone()),
                    Line::from("Channel: ").style(Color::Green),
                    Line::from(self.search_results.items[index].channel.clone()),
                    Line::from("Views: ").style(Color::Green),
                    Line::from(self.search_results.items[index].view_count.clone()),
                    Line::from("Publish time: ").style(Color::Green),
                    Line::from(self.search_results.items[index].publish_time.clone()),
                    Line::from("Duration: ").style(Color::Green),
                    Line::from(self.search_results.items[index].duration.clone()),
                    Line::from("Link: ").style(Color::Green),
                    Line::from(self.search_results.items[index].link.clone()),
                    Line::from("Description: ").style(Color::Green),
                    Line::from(self.search_results.items[index].description.clone()),
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

    // TODO: Render the channel videos list
    fn render_subs(&mut self, area: Rect, buffer: &mut Buffer) {
        let subs_layout = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ]);

        let [channels, feed] = subs_layout.areas(area);

        Block::bordered()
            .title("Channels")
            .render(channels, buffer);

        // Widget::render(
        //     List::new(vec!["Tsoding"]).block(channels_block),
        //     channels, buffer
        // );

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
