use crate::collector::SharedState;
use crate::input::ActiveTab;
use crate::ui::layout::processes::SortCol;

/// All mutable UI state lives here (separate from the data layer).
pub struct AppState {
    pub state:          SharedState,
    pub active_tab:     ActiveTab,
    pub sort_col:       SortCol,
    pub filter_mode:    bool,
    pub filter_text:    String,
    pub scroll_offset:  usize,
    pub selected_proc:  usize,
    /// When true, a second `k` kills with SIGTERM; `K` kills with SIGKILL
    pub kill_confirm:   bool,
}

impl AppState {
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            active_tab:    ActiveTab::Overview,
            sort_col:      SortCol::Cpu,
            filter_mode:   false,
            filter_text:   String::new(),
            scroll_offset: 0,
            selected_proc: 0,
            kill_confirm:  false,
        }
    }
}
