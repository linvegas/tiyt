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

#[derive(Clone, Copy, PartialEq)]
enum AppTab {
    Search,
    Subs
}

struct App {
    // message: String,
    running: bool,
    selected_tab: AppTab,
    selected_search_row: TableState,
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
            running: true,
            selected_tab: AppTab::Search,
            selected_search_row: TableState::default().with_selected(Some(0)),
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
        terminal.draw(|frame| frame.render_widget(self, frame.size()))?;
        Ok(())
    }

    fn handle_event(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
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
                    KeyCode::Char('1') => self.selected_tab = AppTab::Search,
                    KeyCode::Char('2') => self.selected_tab = AppTab::Subs,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn select_next_search_row(&mut self) {
        let i = match self.selected_search_row.selected() {
            Some(i) => if i >= 30 - 1 {0} else {i + 1},
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

        Paragraph::new("Le hecking tsoding")
            .block(Block::bordered().title("Input"))
            .render(input, buffer);

        let results_block = Block::bordered()
            .title("Results");

        let table_data = [
            Row::new(vec!["3D in TypeScript using Ray Casting", "Tsoding Daily", "video"]),
            Row::new(vec!["Why I Don&#39;t Code in Haskell Anymore?", "Tsoding Daily", "video"]),
            Row::new(vec!["Ok, but can you do this in C?", "Tsoding Daily", "video"]),
            Row::new(vec!["Test Anything with Python", "Tsoding Daily", "video"]),
            Row::new(vec!["Hacking Raylib", "Tsoding Daily", "video"]),
            Row::new(vec!["Cracking Secret Message with C and Computer Vision", "Tsoding Daily", "video"]),
            Row::new(vec!["Will Ada Replace C/C++?", "Tsoding", "video"]),
            Row::new(vec!["Can you actually see more than 30 FPS?", "Tsoding Daily", "video"]),
            Row::new(vec!["Is this the Future of Programming Languages?", "Tsoding Daily", "video"]),
            Row::new(vec!["What Keyboard do I use as a Professional Software Developer", "Tsoding Daily", "video"]),
            Row::new(vec!["Hare Programming Language", "Tsoding Daily", "video"]),
            Row::new(vec!["My Next Video Project (Tula Ep.01)", "Tsoding Daily", "video"]),
            Row::new(vec!["Clean Code and Successful Career in Software Development", "Tsoding Daily", "video"]),
            Row::new(vec!["Why do C Programmers Always Obfuscate Their Code?", "Tsoding Daily", "video"]),
            Row::new(vec!["I fixed Lua", "Tsoding Daily", "video"]),
            Row::new(vec!["I tried React and it Ruined My Life", "Tsoding Daily", "video"]),
            Row::new(vec!["The Most Bizarre and Fascinating Project I&#39;ve seen!", "Tsoding Daily", "video"]),
            Row::new(vec!["OOP in Pure C", "Tsoding Daily", "video"]),
            Row::new(vec!["I made JIT Compiler for Brainf*ck lol", "Tsoding Daily", "video"]),
            Row::new(vec!["This is better than TempleOS", "Tsoding Daily", "video"]),
            Row::new(vec!["GameDev in Assembly?!", "Tsoding Daily", "video"]),
            Row::new(vec!["You don&#39;t need DOM", "Tsoding Daily", "video"]),
            Row::new(vec!["Why is C Compiler So Smart?", "Tsoding Daily", "video"]),
            Row::new(vec!["Writing Garbage Collector in C", "Tsoding Daily", "video"]),
            Row::new(vec!["Mini Excel in C — Part 1", "Tsoding Daily", "video"]),
            Row::new(vec!["Parsing Lisp with Rust (Tula Ep.03)", "Tsoding Daily", "video"]),
            Row::new(vec!["Is C++ better than C?", "Tsoding Daily", "video"]),
            Row::new(vec!["Unreasonable Effectiveness of Abstractions", "Tsoding Daily", "video"]),
            Row::new(vec!["Is John Carmack Right about UI?!", "Tsoding Daily", "video"]),
            Row::new(vec!["Can C actually do Perfect Bézier Curves?", "Tsoding Daily", "video"]),
        ];

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
