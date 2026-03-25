use crate::systemd::TimerInfo;

pub enum ViewMode {
    List,
    Detail,
}

pub struct App {
    pub timers: Vec<TimerInfo>,
    pub mode: ViewMode,
    pub selected_index: usize,
    pub detail_status: String,
    pub detail_logs: String,
    pub should_quit: bool,
    pub error: Option<String>,
    pub detail_scroll: usize,
}

impl App {
    pub fn new() -> App {
        App {
            timers: Vec::new(),
            mode: ViewMode::List,
            selected_index: 0,
            detail_status: String::new(),
            detail_logs: String::new(),
            should_quit: false,
            error: None,
            detail_scroll: 0,
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
    }

    pub fn exit_detail(&mut self) {
        self.mode = ViewMode::List;
        self.detail_status.clear();
        self.detail_logs.clear();
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
}
