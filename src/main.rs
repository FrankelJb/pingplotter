#[allow(dead_code)]
mod util;

use crate::util::{
    event::{Event, Events},
    SinSignal,
};
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::{error::Error, io, process::Command};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Terminal,
};

struct App {
    ping_times: Vec<(f64, f64)>,
    window: (f64, f64),
    current_x: f64,
}

impl App {
    fn new() -> App {
        App {
            ping_times: Vec::new(),
            window: (0.0, 100.0),
            current_x: 0.0,
        }
    }

    fn append_time(&mut self, time: f64) {
        self.ping_times.push((self.current_x, time));
        self.current_x += 1.0;
        if self.current_x > self.window.1 {
            self.ping_times.remove(0);
            self.window.0 += 1.0;
            self.window.1 += 1.0;
        }
    }
}

const PING_PATH: &'static str = "/bin/ping";

fn ping(addr: &str) -> Result<f64, Box<dyn Error>> {
    // TODO: Should we support windows?
    // TODO: where is ping
    let output = Command::new(PING_PATH).arg("-c1").arg(addr).output()?;
    if output.status.success() {
        let text = std::str::from_utf8(&output.stdout).unwrap();
        let time_regex = Regex::new(r".*time=(\d+\.\d+).*")?;
        for line in text.lines() {
            if line.contains("from") {
                if let Some(captures) = time_regex.captures(line) {
                    let time = captures.get(1).unwrap().as_str().parse::<f64>()?;
                    return Ok(time);
                }
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "ping was malformed").into())
}

fn main() -> Result<(), Box<dyn Error>> {
    // println!("{:?}", ping("8.8.8.8"));
    // panic!("at the disco");

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    // App
    let app = Arc::new(Mutex::new(App::new()));

    let app_ref = Arc::clone(&app);

    std::thread::spawn(move || loop {
        if let Ok(time) = ping("8.8.8.8" /*"45.220.23.181"*/) {
            app_ref.lock().unwrap().append_time(time);
        }
    });

    loop {
        terminal.draw(|mut f| {
            // let data = Arc::clone(&app);
            let mut app = app.lock().unwrap();
            let size = f.size();
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);
            // println!("{:?}", chunks);
            let x_labels = [
                format!("{}", app.window.0),
                format!("{}", (app.window.0 + app.window.1) / 2.0),
                format!("{}", app.window.1),
            ];
            let datasets = [Dataset::default()
                .name("ping_times")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(&app.ping_times)];
            let chart = Chart::default()
                .block(
                    Block::default()
                        .title("Ping Time")
                        .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("X Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds([app.window.0, app.window.1])
                        .labels(&x_labels),
                )
                .y_axis(
                    Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds([0.0, 1000.0])
                        .labels(&["0", "1000"]),
                )
                .datasets(&datasets);
            f.render_widget(chart, chunks[0]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
                    break;
                }
            }
            Event::Tick => {}
        }
    }

    Ok(())
}
