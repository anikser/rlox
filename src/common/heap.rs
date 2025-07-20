use std::cell::{Cell, RefCell};
use std::fmt;

use super::table::Hashable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectRef(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GcColor {
    White,
    Gray,
    Black,
}

pub struct GcHeader {
    marked: Cell<GcColor>,
    next_gc: Cell<Option<usize>>,
}

pub struct GcObject {
    header: GcHeader,
    pub data: ObjectData,
}

pub enum ObjectData {
    String(ObjString),
}

pub struct ObjString {
    hash: u32,
    data: Vec<u8>,
}

impl ObjString {
    pub fn new(s: &str) -> Self {
        Self {
            hash: Self::calc_hash(s),
            data: s.as_bytes().to_vec(),
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.data) }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn hash(&self) -> u32 {
        self.hash
    }

    fn calc_hash(key: &str) -> u32 {
        let mut hash: u32 = 2166136261;
        for x in key.as_bytes() {
            hash ^= *x as u32;
            hash = hash.wrapping_mul(16777619);
        }
        hash
    }
}

impl Hashable for ObjectRef {
    fn hash(&self) -> u32 {
        self.0 as u32
    }
}

impl fmt::Display for ObjectData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectData::String(s) => write!(f, "{}", s.as_str()),
        }
    }
}

pub struct ObjectHeap {
    objects: RefCell<Vec<Option<GcObject>>>,
    free_list: RefCell<Vec<usize>>,
    bytes_allocated: Cell<usize>,
    next_gc: Cell<usize>,
    gc_roots: RefCell<Vec<ObjectRef>>,
}

const HEAP_GROW_FACTOR: usize = 2;
const INITIAL_GC_THRESHOLD: usize = 1024 * 1024; // 1MB

impl ObjectHeap {
    pub fn new() -> Self {
        Self {
            objects: RefCell::new(Vec::new()),
            free_list: RefCell::new(Vec::new()),
            bytes_allocated: Cell::new(0),
            next_gc: Cell::new(INITIAL_GC_THRESHOLD),
            gc_roots: RefCell::new(Vec::new()),
        }
    }

    pub fn alloc_string(&self, s: &str) -> ObjectRef {
        let size = std::mem::size_of::<GcObject>() + s.len();
        self.maybe_gc(size);
        
        let obj = GcObject {
            header: GcHeader {
                marked: Cell::new(GcColor::White),
                next_gc: Cell::new(None),
            },
            data: ObjectData::String(ObjString::new(s)),
        };
        
        self.alloc_object(obj, size)
    }

    fn alloc_object(&self, obj: GcObject, size: usize) -> ObjectRef {
        let mut objects = self.objects.borrow_mut();
        let mut free_list = self.free_list.borrow_mut();
        
        let index = if let Some(free_index) = free_list.pop() {
            objects[free_index] = Some(obj);
            free_index
        } else {
            objects.push(Some(obj));
            objects.len() - 1
        };
        
        self.bytes_allocated.set(self.bytes_allocated.get() + size);
        ObjectRef(index)
    }

    pub fn get(&self, obj_ref: ObjectRef) -> Option<std::cell::Ref<'_, GcObject>> {
        std::cell::Ref::filter_map(self.objects.borrow(), |objects| {
            objects.get(obj_ref.0)?.as_ref()
        }).ok()
    }

    pub fn get_string(&self, obj_ref: ObjectRef) -> Option<String> {
        self.get(obj_ref).and_then(|obj| match &obj.data {
            ObjectData::String(s) => Some(s.as_str().to_string()),
        })
    }
    
    pub fn get_string_hash(&self, obj_ref: ObjectRef) -> Option<u32> {
        self.get(obj_ref).and_then(|obj| match &obj.data {
            ObjectData::String(s) => Some(s.hash()),
        })
    }

    pub fn add_root(&self, obj: ObjectRef) {
        self.gc_roots.borrow_mut().push(obj);
    }

    pub fn remove_root(&self, obj: ObjectRef) {
        self.gc_roots.borrow_mut().retain(|&r| r != obj);
    }

    fn maybe_gc(&self, additional_bytes: usize) {
        if self.bytes_allocated.get() + additional_bytes > self.next_gc.get() {
            self.collect_garbage();
        }
    }

    pub fn collect_garbage(&self) {
        self.mark_roots();
        self.trace_references();
        self.sweep();
        
        self.next_gc.set(self.bytes_allocated.get() * HEAP_GROW_FACTOR);
    }

    fn mark_roots(&self) {
        for &root in self.gc_roots.borrow().iter() {
            self.mark_object(root);
        }
    }

    fn mark_object(&self, obj_ref: ObjectRef) {
        if let Some(obj) = self.objects.borrow().get(obj_ref.0).and_then(|o| o.as_ref()) {
            if obj.header.marked.get() == GcColor::White {
                obj.header.marked.set(GcColor::Gray);
            }
        }
    }

    fn trace_references(&self) {
        loop {
            let gray_object = self.find_gray_object();
            match gray_object {
                Some(obj_ref) => {
                    self.blacken_object(obj_ref);
                }
                None => break,
            }
        }
    }

    fn find_gray_object(&self) -> Option<ObjectRef> {
        let objects = self.objects.borrow();
        for (index, obj_opt) in objects.iter().enumerate() {
            if let Some(obj) = obj_opt {
                if obj.header.marked.get() == GcColor::Gray {
                    return Some(ObjectRef(index));
                }
            }
        }
        None
    }

    fn blacken_object(&self, obj_ref: ObjectRef) {
        if let Some(obj) = self.objects.borrow().get(obj_ref.0).and_then(|o| o.as_ref()) {
            obj.header.marked.set(GcColor::Black);
            // Note: strings don't reference other objects, so no need to trace further
        }
    }

    fn sweep(&self) {
        let mut objects = self.objects.borrow_mut();
        let mut free_list = self.free_list.borrow_mut();
        let mut bytes_freed = 0;

        for (index, obj_opt) in objects.iter_mut().enumerate() {
            if let Some(obj) = obj_opt {
                if obj.header.marked.get() == GcColor::White {
                    bytes_freed += match &obj.data {
                        ObjectData::String(s) => std::mem::size_of::<GcObject>() + s.len(),
                    };
                    *obj_opt = None;
                    free_list.push(index);
                } else {
                    obj.header.marked.set(GcColor::White);
                }
            }
        }

        self.bytes_allocated.set(self.bytes_allocated.get().saturating_sub(bytes_freed));
    }

    pub fn stats(&self) -> HeapStats {
        let objects = self.objects.borrow();
        let live_objects = objects.iter().filter(|o| o.is_some()).count();
        HeapStats {
            bytes_allocated: self.bytes_allocated.get(),
            next_gc: self.next_gc.get(),
            total_slots: objects.len(),
            live_objects,
            free_slots: self.free_list.borrow().len(),
        }
    }
}

pub struct HeapStats {
    pub bytes_allocated: usize,
    pub next_gc: usize,
    pub total_slots: usize,
    pub live_objects: usize,
    pub free_slots: usize,
}