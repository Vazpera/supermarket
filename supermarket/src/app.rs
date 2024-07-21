use std::io;

use crate::tui;
use ratatui::widgets::{self, Gauge};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Layout,
    layout::Rect,
    prelude::Constraint,
    prelude::Direction,
    prelude::Style,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Row, Widget},
    Frame,
};
use sysinfo::System;

#[derive(Clone, Copy)]
pub struct App {
    exit: bool,
    curr: usize,
}

impl AsMut<App> for App {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a> Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            curr: 0,
        }
    }
}

impl<'a> Widget for App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Please note that we use "new_all" to ensure that all list of
        // components, network interfaces, disks and users are already
        // filled!
        let mut sys = System::new_all();

        // First we update all information of our `System` struct.
        sys.refresh_all();
        let usage = sys.global_cpu_info().cpu_usage();

        let layout = Layout::new(
            Direction::Horizontal,
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(area);
        let right_side = Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(13),
                Constraint::Fill(100),
            ],
        )
        .split(layout[1]);

        let memory_bar = Gauge::default()
            .percent((100.0 * (sys.used_memory() as f32 / sys.total_memory() as f32)) as u16)
            .gauge_style(Style::new().red())
            .use_unicode(true)
            .label(format!(
                "{}%",
                (100.0 * (sys.used_memory() as f32 / sys.total_memory() as f32)).round()
            ))
            .block(Block::bordered().title("Used Memory").on_black());
        memory_bar.render(right_side[0], buf);

        let usage_bar = Gauge::default()
            .percent((usage) as u16)
            .gauge_style(Style::new().red())
            .use_unicode(true)
            .label(format!("{}%", (usage.round())))
            .block(Block::bordered().title("CPU Usage").on_black());
        usage_bar.render(right_side[1], buf);

        let mut procs: Vec<Constraint> = vec![];
        let mut i = 0;
        let mut sorted: Vec<(&sysinfo::Pid, &sysinfo::Process)> = sys.processes().iter().collect();
        sorted.sort_unstable_by(|a, b| (b.1.memory()).cmp(&a.1.memory()));
        let mut rows: Vec<Row> = vec![];
        rows.push(Row::new(vec!["Name", "Memory Usage"]).black().on_red());
        for (_, process) in sorted {
            if i >= 10 {
                break;
            }
            procs.push(Constraint::Max(3));
            rows.push(Row::new(vec![
                process.name().to_string(),
                format!("{}%", 100 * process.memory() / sys.total_memory()),
            ]));
            match i % 2 {
                0 => rows[i + 1] = rows[i + 1].clone().on_black(),
                _ => {}
            }
            i += 1;
        }

        let table = widgets::Table::new(
            rows,
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(Block::bordered().title("Processes").on_black());
        table.render(right_side[2], buf);

        let left_side = Layout::new(
            Direction::Vertical,
            vec![Constraint::Max(6), Constraint::Fill(100)],
        )
        .split(layout[0]);

        let paragraph = Paragraph::new(vec![
            Line::from(vec![
                "System Name: ".red().bold(),
                System::name().expect("Name not found!").red(),
            ]),
            Line::from(vec![
                "Host Name: ".red().bold(),
                System::host_name().expect("Name not found!").red(),
            ]),
            Line::from(vec![
                "OS Version: ".red().bold(),
                System::os_version().expect("Name not found!").red(),
            ]),
            Line::from(vec![
                "Kernel Version: ".red().bold(),
                System::kernel_version().expect("Name not found!").red(),
            ]),
        ])
        .block(Block::bordered().title("System Information").on_black());
        paragraph.render(left_side[0], buf);

        let mut specs: Vec<Line> = vec![];
        specs.push(Line::from(vec![
            "Core Count: ".red().bold(),
            format!("{}", sys.physical_core_count().expect("No Cores!")).red(),
        ]));
        specs.push(Line::from(vec![
            "Total RAM: ".red().bold(),
            format!(
                "{:.2} GB",
                sys.total_memory() as f32 / ((2.0 as f32).powf(30.0))
            )
            .red(),
        ]));

        let spec_table = Paragraph::new(specs)
            .block(Block::bordered().title("Specs"))
            .on_black();
        spec_table.render(left_side[1], buf);
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(*self, frame.size());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (key_event.code, self.curr) {
            (KeyCode::Char('q'), _) => self.exit = true,
            (KeyCode::Left, _) => self.curr = (self.curr + 2) % 3,
            (KeyCode::Right, _) => self.curr = (self.curr + 1) % 3,

            _ => {}
        }
    }
}
