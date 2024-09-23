use {
    matetui::{component, Component},
    ratatui::{
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::Stylize,
        widgets::{block::Title, Block},
    },
    std::time::Instant,
};

component! {
    pub struct FpsComponent {
        app_start_time: Option<Instant>,
        render_start_time: Option<Instant>,
        app_frames: u32,
        app_fps: f64,
        render_frames: u32,
        render_fps: f64,
    }
}

impl FpsComponent {
    pub fn new() -> Self {
        Self {
            app_start_time: Some(Instant::now()),
            render_start_time: Some(Instant::now()),
            ..Self::default()
        }
    }
}

impl Component for FpsComponent {
    fn handle_frame_event(&mut self) -> Option<matetui::Action> {
        if let Some(render_start_time) = self.render_start_time {
            self.render_frames += 1;
            let now = Instant::now();
            let elapsed = (now - render_start_time).as_secs_f64();
            if elapsed >= 1.0 {
                self.render_fps = self.render_frames as f64 / elapsed;
                self.render_start_time = Some(now);
                self.render_frames = 0;
            }
        }
        None
    }

    fn handle_tick_event(&mut self) -> Option<matetui::Action> {
        if let Some(app_start_time) = self.app_start_time {
            self.app_frames += 1;
            let now = Instant::now();
            let elapsed = (now - app_start_time).as_secs_f64();
            if elapsed >= 1.0 {
                self.app_fps = self.app_frames as f64 / elapsed;
                self.app_start_time = Some(now);
                self.app_frames = 0;
            }
        }
        None
    }

    fn draw(&mut self, f: &mut matetui::Frame<'_>, area: Rect) {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let rect = rects[0];

        let s = format!(
            "{:.2} ticks per sec (app) / {:.2} frames per sec (render)",
            self.app_fps, self.render_fps
        );
        let block = Block::default().title(Title::from(s.dim()).alignment(Alignment::Right));
        f.render_widget(block, rect);
    }
}
