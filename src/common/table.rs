pub trait Hashable {
    fn hash(&self) -> u32;
}
pub struct Table<K, V>
where
    K: Hashable,
{
    count: u32,
    capacity: usize,
    entries: Vec<Option<_Entry<Entry<K, V>>>>,
}

struct Entry<K, V>
where
    K: Hashable,
{
    key: K,
    value: V,
}

enum _Entry<T> {
    Tombstone,
    Some(T),
}

impl<K, V> Table<K, V>
where
    K: Hashable + PartialEq + Clone,
{
    const MAX_LOAD: f64 = 0.75;
    const GROW_FACTOR: usize = 2;

    pub fn new() -> Self {
        Table {
            count: 0,
            capacity: 0,
            entries: Vec::new(),
        }
    }

    pub fn set(&mut self, key: &K, value: V) -> bool {
        if (self.count + 1) as f64 > self.capacity as f64 * Self::MAX_LOAD {
            self.grow_capacity(self.capacity * Self::GROW_FACTOR);
        }

        let index = Self::find_entry_idx(&self.entries, self.capacity, &key);
        let is_new = matches!(self.entries[index], None);
        let is_tombstone = matches!(self.entries[index], Some(_Entry::Tombstone));
        self.entries[index] = Some(_Entry::Some(Entry {
            key: key.clone(),
            value,
        }));
        if is_new {
            self.count += 1;
        }
        is_new || is_tombstone
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let idx = Self::find_entry_idx(&self.entries, self.capacity, key);
        match &self.entries[idx] {
            Some(_Entry::Some(entry)) => Some(&entry.value),
            None => None,
            Some(_Entry::Tombstone) => None,
        }
    }

    pub fn delete(&mut self, key: &K) -> bool {
        let idx = Self::find_entry_idx(&self.entries, self.capacity, key);
        let exists = matches!(self.entries[idx], Some(_Entry::Some(_)));

        if exists {
            self.entries[idx] = Some(_Entry::Tombstone);
        }

        exists
    }

    fn find_entry_idx(
        entries: &Vec<Option<_Entry<Entry<K, V>>>>,
        capacity: usize,
        key: &K,
    ) -> usize {
        let mut index = key.hash() as usize % capacity;
        let mut tombstone_idx = None;
        loop {
            let entry = &entries[index];
            match entry {
                None => {
                    return match tombstone_idx {
                        Some(idx) => idx,
                        None => index,
                    }
                }
                Some(_Entry::Some(entry)) if (&entry.key).eq(key) => return index,
                Some(_Entry::Tombstone) => tombstone_idx = Some(index),
                _ => (),
            }

            index = (index + 1) % capacity;
        }
    }

    fn grow_capacity(&mut self, capacity: usize) {
        let mut entries: Vec<Option<_Entry<Entry<K, V>>>> = Vec::with_capacity(capacity);
        for i in 0..capacity {
            entries[i] = None;
        }

        for i in 0..self.capacity {
            if let Some(entry) = self.entries[i].take() {
                match entry {
                    _Entry::Some(entry) => {
                        let idx = Self::find_entry_idx(&entries, capacity, &entry.key);
                        entries[idx] = Some(_Entry::Some(entry));
                    }
                    _ => (),
                }
            }
        }

        self.entries = entries;
        self.capacity = capacity;
    }
}
