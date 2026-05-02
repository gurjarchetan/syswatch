use std::cmp::Ordering;
use crate::collector::SharedState;
use crate::collector::process::{ProcessInfo, ProcSummary};
use crate::input::ActiveTab;
use crate::ui::layout::processes::SortCol;

/// All mutable UI state lives here (separate from the data layer).
pub struct AppState {
    pub state:            SharedState,
    pub active_tab:       ActiveTab,
    pub sort_col:         SortCol,
    pub filter_mode:      bool,
    pub filter_text:      String,
    pub scroll_offset:    usize,
    pub selected_proc:    usize,
    /// When true, a second `k` kills with SIGTERM; `K` kills with SIGKILL
    pub kill_confirm:     bool,
    /// When true, show detail popup for selected process in F2
    pub show_proc_detail: bool,
    /// PID captured at the moment the detail popup was opened (stable across re-sorts)
    pub detail_pid: Option<u32>,

    // ── Process render cache ──────────────────────────────────────────────────
    /// Sorted + filtered process list, ready to render — rebuilt only when the
    /// underlying data, sort column, or filter text changes (~1 s cadence).
    pub proc_cache:         Vec<ProcessInfo>,
    pub proc_summary_cache: ProcSummary,
    proc_cache_gen:    u64,
    proc_cache_sort:   SortCol,
    proc_cache_filter: String,
    /// Set to true when the display needs to be redrawn (user input, popup toggle, etc.).
    /// Cleared after each draw. The render loop also redraws every 500ms regardless.
    pub needs_redraw: bool,
}

impl AppState {
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            active_tab:       ActiveTab::Overview,
            sort_col:         SortCol::Cpu,
            filter_mode:      false,
            filter_text:      String::new(),
            scroll_offset:    0,
            selected_proc:    0,
            kill_confirm:     false,
            show_proc_detail: false,
            detail_pid:       None,
            proc_cache:         Vec::new(),
            proc_summary_cache: ProcSummary::default(),
            proc_cache_gen:    u64::MAX, // force first build
            proc_cache_sort:   SortCol::Cpu,
            proc_cache_filter: String::new(),
            needs_redraw:      true,
        }
    }

    /// Rebuild `proc_cache` and `proc_summary_cache` only when the underlying
    /// data, sort column, or filter text has changed since the last build.
    /// Call this once per render loop iteration, before `terminal.draw()`.
    /// Returns `true` if the cache was actually rebuilt (data or sort changed).
    pub fn update_proc_cache(&mut self) -> bool {
        // Fast path: check staleness under a brief read lock.
        let (gen, needs_rebuild) = {
            let s = self.state.read();
            let stale = s.proc_gen       != self.proc_cache_gen
                || self.sort_col         != self.proc_cache_sort
                || self.filter_text      != self.proc_cache_filter;
            (s.proc_gen, stale)
        };
        if !needs_rebuild { return false; }

        // Slow path: clone + filter + sort (~1 s cadence or on user interaction).
        let (mut procs, summary) = {
            let s = self.state.read();
            (s.processes.clone(), s.proc_summary.clone())
        };
        if !self.filter_text.is_empty() {
            let q = self.filter_text.to_lowercase();
            procs.retain(|p| p.name.to_lowercase().contains(&q));
        }
        match self.sort_col {
            SortCol::Cpu  => procs.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(Ordering::Equal)),
            SortCol::Mem  => procs.sort_by(|a, b| b.mem_pct.partial_cmp(&a.mem_pct).unwrap_or(Ordering::Equal)),
            SortCol::Pid  => procs.sort_by_key(|p| p.pid),
            SortCol::Name => procs.sort_by(|a, b| a.name.cmp(&b.name)),
        }
        self.proc_cache         = procs;
        self.proc_summary_cache = summary;
        self.proc_cache_gen     = gen;
        self.proc_cache_sort    = self.sort_col;
        self.proc_cache_filter  = self.filter_text.clone();
        true
    }
}
