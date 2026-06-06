use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub enum Event {
    ForwardClock(u32),
}

#[derive(Default)]
pub struct EventBus {
    queue: Rc<RefCell<VecDeque<Event>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_event(&self) -> Option<Event> {
        self.queue.borrow_mut().pop_front()
    }

    pub fn new_writer(&self) -> EventWriter {
        EventWriter {
            queue: self.queue.clone(),
        }
    }
}

pub struct EventWriter {
    queue: Rc<RefCell<VecDeque<Event>>>,
}

impl EventWriter {
    pub fn push_event(&self, event: Event) {
        self.queue.borrow_mut().push_back(event);
    }
}
