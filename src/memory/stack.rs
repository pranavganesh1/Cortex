use super::context::ContextFrame;
use chrono::Utc;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use uuid::Uuid;

pub static WORKING_MEMORY: Lazy<Mutex<WorkingMemory>> = Lazy::new(|| {
    Mutex::new(WorkingMemory::new())
});

pub struct WorkingMemory {
    stack: Vec<ContextFrame>,
    current: ContextFrame,
    max_stack_depth: usize,
    context_switch_threshold_minutes: i64,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            max_stack_depth: 10,
            current: ContextFrame::new(format!("ctx-{}", Uuid::new_v4())),
            context_switch_threshold_minutes: 15,
        }
    }

    pub fn touch_file(&mut self, path: &str) {
        let now = Utc::now();

        // Check if file belongs to current context
        if self.current.is_related_to_file(path) {
            self.current.add_file(path);
            return;
        }

        // File is unrelated — check if current context is stale
        if self.current.is_stale(now, self.context_switch_threshold_minutes) {
            // Context switch: push current, start new
            self.push_current();
            self.current = ContextFrame::new(format!("ctx-{}", Uuid::new_v4()));
            self.current.add_file(path);
        } else {
            // Still active but different area — could be expanding context
            // or a brief detour. Add it and see.
            self.current.add_file(path);
            // If after adding, the topic changed significantly, we might
            // want to split. For now, keep it simple.
        }
    }

    pub fn add_decision(&mut self, text: &str) {
        self.current.add_decision(text);
    }

    pub fn add_query(&mut self, text: &str) {
        self.current.add_query(text);
    }

    pub fn add_commit(&mut self, hash: &str) {
        self.current.add_commit(hash);
    }

    pub fn push_current(&mut self) {
        let mut frame = std::mem::replace(
            &mut self.current,
            ContextFrame::new(format!("ctx-{}", Uuid::new_v4()))
        );
        frame.is_active = false;
        self.stack.push(frame);
        if self.stack.len() > self.max_stack_depth {
            self.stack.remove(0);
        }
    }

    pub fn back(&mut self) -> Option<ContextFrame> {
        if let Some(prev) = self.stack.pop() {
            let mut old_current = std::mem::replace(&mut self.current, prev);
            old_current.is_active = false;
            self.current.is_active = true;
            self.current.last_active = Utc::now();
            Some(old_current)
        } else {
            None
        }
    }

    pub fn current(&self) -> &ContextFrame {
        &self.current
    }

    pub fn stack(&self) -> &[ContextFrame] {
        &self.stack
    }

    pub fn all_contexts(&self) -> Vec<&ContextFrame> {
        let mut all = Vec::new();
        for ctx in &self.stack {
            all.push(ctx);
        }
        all.push(&self.current);
        all
    }

    pub fn find_context_by_topic(&self, topic: &str) -> Option<&ContextFrame> {
        self.all_contexts().into_iter().rev().find(|c| {
            c.topic.to_lowercase().contains(&topic.to_lowercase()) ||
            topic.to_lowercase().contains(&c.topic.to_lowercase())
        })
    }

    pub fn recent_contexts(&self, n: usize) -> Vec<&ContextFrame> {
        self.all_contexts().into_iter().rev().take(n).collect()
    }
}

// Thread-safe helpers
pub fn touch_file(path: &str) {
    if let Ok(mut wm) = WORKING_MEMORY.lock() {
        wm.touch_file(path);
    }
}

pub fn add_decision(text: &str) {
    if let Ok(mut wm) = WORKING_MEMORY.lock() {
        wm.add_decision(text);
    }
}

pub fn add_query(text: &str) {
    if let Ok(mut wm) = WORKING_MEMORY.lock() {
        wm.add_query(text);
    }
}

pub fn add_commit(hash: &str) {
    if let Ok(mut wm) = WORKING_MEMORY.lock() {
        wm.add_commit(hash);
    }
}

pub fn with_current<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&ContextFrame) -> R,
{
    WORKING_MEMORY.lock().ok().map(|wm| f(wm.current()))
}

pub fn with_stack<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&WorkingMemory) -> R,
{
    WORKING_MEMORY.lock().ok().map(|wm| f(&*wm))
}

pub fn back() -> Option<ContextFrame> {
    WORKING_MEMORY.lock().ok()?.back()
}
