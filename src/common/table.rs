use super::{ObjString, Value};

struct Table {
    count: u32,
    capacity: usize,
    entries: Vec<Option<Entry>>,
}

struct Entry {
    key: Box<ObjString>,
    value: Value,
}

impl Table {
    const MAX_LOAD: f64 = 0.75;

    pub fn new() -> Self {
        Table {
            count: 0,
            capacity: 0,
            entries: Vec::new(),
        }
    }

    pub fn set(&mut self, key: Box<ObjString>, value: Value) -> bool {
        if (self.count + 1) as f64 > self.capacity as f64 * Self::MAX_LOAD {
            self.grow_capacity(self.capacity * 2);
        }

        let index = Self::find_entry_idx(&self.entries, self.capacity, &key);
        let is_new = matches!(self.entries[index], None);
        self.entries[index] = Some(Entry {
            key: key,
            value: value,
        });
        is_new
    }

    pub fn get(&self, key:Box<ObjString>) -> Option<{

    }

    fn find_entry_idx(
        entries: &Vec<Option<Entry>>,
        capacity: usize,
        key: &Box<ObjString>,
    ) -> usize {
        let mut index = key.hash as usize % capacity;
        loop {
            let entry = &entries[index];
            match entry {
                None => return index,
                Some(entry) if entry.key == *key => return index,
                _ => (),
            }

            index = (index + 1) % capacity;
        }
    }

    fn grow_capacity(&mut self, capacity: usize) {
        let mut entries: Vec<Option<Entry>> = Vec::with_capacity(capacity);
        for i in 0..capacity {
            entries[i] = None;
        }

        for i in 0..self.capacity {
            if let Some(entry) = self.entries[i].take() {
                let idx = Self::find_entry_idx(&entries, capacity, &entry.key);
                entries[idx] = Some(entry);
            }
        }

        self.entries = entries;
        self.capacity = capacity;
    }
}
