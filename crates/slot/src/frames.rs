use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct Frames {
    inner: Mutex<Inner>,
    size: usize,
    dims: (u32, u32),
    shown: AtomicU64,
    taken: AtomicU64,
}

struct Inner {
    ready: Option<Vec<u8>>,
    spare: Vec<Vec<u8>>,
    allocated: usize,
}

impl Frames {
    pub fn new(size: usize) -> Arc<Self> {
        Self::build(size, (240, 160))
    }

    /// Frames for a video buffer of `w` x `h` pixels.
    pub fn sized(w: u32, h: u32) -> Arc<Self> {
        Self::build((w * h * 4) as usize, (w, h))
    }

    fn build(size: usize, dims: (u32, u32)) -> Arc<Self> {
        Arc::new(Frames {
            inner: Mutex::new(Inner {
                ready: None,
                spare: Vec::new(),
                allocated: 0,
            }),
            size,
            dims,
            shown: AtomicU64::new(pack(dims)),
            taken: AtomicU64::new(0),
        })
    }

    pub fn dims(&self) -> (u32, u32) {
        self.dims
    }

    pub fn shown(&self) -> (u32, u32) {
        unpack(self.shown.load(Ordering::Relaxed))
    }

    pub fn set_shown(&self, shown: (u32, u32)) {
        self.shown.store(pack(shown), Ordering::Relaxed);
    }

    pub fn take_write(&self) -> Vec<u8> {
        let mut i = self.lock();
        match i.spare.pop() {
            Some(buf) => buf,
            None => {
                i.allocated += 1;
                Vec::with_capacity(self.size)
            }
        }
    }

    pub fn publish(&self, buf: Vec<u8>) {
        let mut i = self.lock();
        if let Some(dropped) = i.ready.replace(buf) {
            i.spare.push(dropped);
        }
    }

    pub fn latest(self: &Arc<Self>) -> Option<FrameRef> {
        let buf = self.lock().ready.take()?;
        self.taken.fetch_add(1, Ordering::Relaxed);
        Some(FrameRef {
            frames: self.clone(),
            buf,
        })
    }

    pub fn is_ready(&self) -> bool {
        self.lock().ready.is_some()
    }

    pub fn taken(&self) -> u64 {
        self.taken.load(Ordering::Relaxed)
    }

    pub fn allocated(&self) -> usize {
        self.lock().allocated
    }

    fn recycle(&self, buf: Vec<u8>) {
        self.lock().spare.push(buf);
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn pack((w, h): (u32, u32)) -> u64 {
    (w as u64) << 32 | h as u64
}

fn unpack(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, v as u32)
}

pub struct FrameRef {
    frames: Arc<Frames>,
    buf: Vec<u8>,
}

impl FrameRef {
    /// Size of the buffer this frame is in.
    pub fn dims(&self) -> (u32, u32) {
        self.frames.dims()
    }

    /// Size of the picture the core drew inside the buffer.
    pub fn shown(&self) -> (u32, u32) {
        self.frames.shown()
    }
}

impl Deref for FrameRef {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.buf
    }
}

impl Drop for FrameRef {
    fn drop(&mut self) {
        self.frames.recycle(std::mem::take(&mut self.buf));
    }
}
