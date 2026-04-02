use crate::systemd::TimerInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailPaneFocus {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailContentMode {
    Logs,
    ServiceFile,
}

pub struct App {
    pub timers: Vec<TimerInfo>,
    pub mode: ViewMode,
    pub selected_index: usize,
    pub detail_status: String,
    pub detail_logs: String,
    pub detail_focus: DetailPaneFocus,
    pub detail_content_mode: DetailContentMode,
    pub should_quit: bool,
    pub error: Option<String>,
    pub detail_scroll: usize,
    pub auto_scroll: bool,
}

impl App {
    pub fn new() -> App {
        App {
            timers: Vec::new(),
            mode: ViewMode::List,
            selected_index: 0,
            detail_status: String::new(),
            detail_logs: String::new(),
            detail_focus: DetailPaneFocus::Top,
            detail_content_mode: DetailContentMode::Logs,
            should_quit: false,
            error: None,
            detail_scroll: 0,
            auto_scroll: true,
        }
    }

    pub fn next(&mut self) {
        if self.timers.is_empty() {
            return;
        }
        let i = match self.selected_index {
            _ if self.selected_index >= self.timers.len() - 1 => 0,
            _ => self.selected_index + 1,
        };
        self.selected_index = i;
    }

    pub fn previous(&mut self) {
        if self.timers.is_empty() {
            return;
        }
        let i = match self.selected_index {
            0 => self.timers.len() - 1,
            _ => self.selected_index - 1,
        };
        self.selected_index = i;
    }

    pub fn enter_detail(&mut self) {
        self.mode = ViewMode::Detail;
        self.detail_focus = DetailPaneFocus::Top;
        self.detail_content_mode = DetailContentMode::Logs;
        self.detail_scroll = 0;
        self.auto_scroll = true;
    }

    pub fn exit_detail(&mut self) {
        self.mode = ViewMode::List;
        self.detail_status.clear();
        self.detail_logs.clear();
        self.detail_focus = DetailPaneFocus::Top;
        self.detail_content_mode = DetailContentMode::Logs;
        self.detail_scroll = 0;
        self.auto_scroll = true;
    }

    pub fn toggle_detail_focus(&mut self) {
        self.detail_focus = match self.detail_focus {
            DetailPaneFocus::Top => DetailPaneFocus::Bottom,
            DetailPaneFocus::Bottom => DetailPaneFocus::Top,
        };
    }

    pub fn select_next_detail_content(&mut self) {
        self.detail_content_mode = match self.detail_content_mode {
            DetailContentMode::Logs => DetailContentMode::ServiceFile,
            DetailContentMode::ServiceFile => DetailContentMode::Logs,
        };
    }

    pub fn select_previous_detail_content(&mut self) {
        self.select_next_detail_content();
    }

    pub fn scroll_detail_down(&mut self, max_lines: usize) {
        if self.detail_scroll < max_lines {
            self.detail_scroll += 1;
        }
        
        if self.detail_scroll >= max_lines {
            self.auto_scroll = true;
        }
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
        self.auto_scroll = false;
    }

    pub fn selected_timer(&self) -> Option<&TimerInfo> {
        self.timers.get(self.selected_index)
    }
}

#[cfg(test)]
mod tests {
    use super::{App, DetailContentMode, DetailPaneFocus};

    #[test]
    fn detail_view_defaults_to_logs_mode() {
        let mut app = App::new();

        app.enter_detail();

        assert!(matches!(app.mode, super::ViewMode::Detail));
        assert_eq!(app.detail_focus, DetailPaneFocus::Top);
        assert_eq!(app.detail_content_mode, DetailContentMode::Logs);
        assert_eq!(app.detail_scroll, 0);
    }

    #[test]
    fn detail_focus_toggles_between_panes() {
        let mut app = App::new();

        app.enter_detail();
        app.toggle_detail_focus();
        assert_eq!(app.detail_focus, DetailPaneFocus::Bottom);

        app.toggle_detail_focus();
        assert_eq!(app.detail_focus, DetailPaneFocus::Top);
    }

    #[test]
    fn detail_content_mode_switches_between_logs_and_service_file() {
        let mut app = App::new();

        app.enter_detail();
        app.select_next_detail_content();
        assert_eq!(app.detail_content_mode, DetailContentMode::ServiceFile);

        app.select_previous_detail_content();
        assert_eq!(app.detail_content_mode, DetailContentMode::Logs);
    }

    #[test]
    fn detail_scroll_helpers_clamp_to_valid_bounds() {
        let mut app = App::new();

        app.enter_detail();
        app.scroll_detail_down(3);
        assert_eq!(app.detail_scroll, 1);

        app.scroll_detail_down(3);
        app.scroll_detail_down(3);
        app.scroll_detail_down(3);
        assert_eq!(app.detail_scroll, 3);

        app.scroll_detail_up();
        assert_eq!(app.detail_scroll, 2);

        app.scroll_detail_up();
        app.scroll_detail_up();
        app.scroll_detail_up();
        assert_eq!(app.detail_scroll, 0);
    }

    #[test]
    fn detail_scroll_manages_auto_scroll_state() {
        let mut app = App::new();

        app.enter_detail();
        assert!(app.auto_scroll);

        // Scrolling up disables auto-scroll
        app.scroll_detail_up();
        assert!(!app.auto_scroll);

        // Scrolling down to the max (simulated bottom) re-enables auto-scroll
        app.scroll_detail_down(5); // scroll=0
        app.scroll_detail_down(5); // scroll=1
        app.scroll_detail_down(5); // scroll=2
        app.scroll_detail_down(5); // scroll=3
        app.scroll_detail_down(5); // scroll=4
        app.scroll_detail_down(5); // scroll=5
        assert!(app.auto_scroll);
    }
}
