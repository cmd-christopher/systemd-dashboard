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
