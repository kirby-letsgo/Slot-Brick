use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use slot_gfx::{Compositor, TexId};
use slot_store::Cart;
use slot_ui::{cart_face_with, CartFace, ShelfBacking};

use crate::app::{App, FaceWant};
use crate::label_cache::label_art;

const WORKERS: usize = 2;

type Key = (u64, usize, usize);

#[derive(Default)]
struct Queue {
    jobs: VecDeque<(Key, Cart)>,
    closed: bool,
}

type Shared = Arc<(Mutex<Queue>, Condvar)>;

pub struct CartFaces {
    queue: Shared,
    built: Receiver<(Key, CartFace, ShelfBacking)>,
    pending: HashSet<Key>,
    asked: Vec<FaceWant>,
    revision: u64,
    free: Vec<TexId>,
}

impl CartFaces {
    pub fn spawn(root: PathBuf) -> Self {
        let queue: Shared = Arc::default();
        let (outbox, built) = mpsc::channel();
        for n in 0..WORKERS {
            let (queue, outbox, root) = (queue.clone(), outbox.clone(), root.clone());
            let spawned = thread::Builder::new()
                .name(format!("slot-carts-{n}"))
                .spawn(move || work(&queue, &outbox, &root));
            if let Err(e) = spawned {
                eprintln!("slot: cart faces: worker thread failed to start: {e}");
            }
        }
        CartFaces {
            queue,
            built,
            pending: HashSet::new(),
            asked: Vec::new(),
            revision: 0,
            free: Vec::new(),
        }
    }

    pub fn sync(&mut self, app: &mut App, compositor: &mut Compositor) {
        self.free.extend(app.shed_faces());
        while let Ok((key, face, backing)) = self.built.try_recv() {
            self.place(app, compositor, key, face, backing);
        }
        let wants = app.face_wants();
        if wants != self.asked || self.revision != app.face_revision() {
            self.revision = app.face_revision();
            self.requeue(app, &wants);
            self.asked = wants;
        }
        let waiting: Vec<Key> = self
            .asked
            .iter()
            .filter(|w| w.on_screen)
            .map(|w| (self.revision, w.shelf, w.index))
            .collect();
        for (key, face, backing) in visible_ready(&self.built, waiting) {
            self.place(app, compositor, key, face, backing);
        }
    }

    fn requeue(&mut self, app: &App, wants: &[FaceWant]) {
        let (lock, wake) = &*self.queue;
        let mut queue = lock.lock().unwrap_or_else(|e| e.into_inner());
        for (key, _) in queue.jobs.drain(..) {
            self.pending.remove(&key);
        }
        for w in wants {
            let key = (self.revision, w.shelf, w.index);
            if self.pending.contains(&key) {
                continue;
            }
            if let Some(cart) = app.face_cart(w.shelf, w.index) {
                queue.jobs.push_back((key, cart.clone()));
                self.pending.insert(key);
            }
        }
        wake.notify_all();
    }

    fn place(&mut self, app: &mut App, compositor: &mut Compositor, key: Key, face: CartFace, backing: ShelfBacking) {
        self.pending.remove(&key);
        // Sorting or a new search can put a different game at the same shelf position
        // while a worker is still rendering the old one.
        if key.0 != app.face_revision() { return; }
        self.asked.retain(|w| (self.revision, w.shelf, w.index) != key);
        let tex = match self.free.pop() {
            Some(tex) => {
                compositor.update_texture(tex, face.w, face.h, &face.rgba);
                tex
            }
            None => compositor.create_texture(face.w, face.h, &face.rgba),
        };
        self.free.extend(app.set_prepared_cart_face(key.1, key.2, tex, backing));
    }
}

/// One shared budget per frame, including unrelated results that finish first.
fn visible_ready(built: &Receiver<(Key, CartFace, ShelfBacking)>, mut waiting: Vec<Key>) -> Vec<(Key, CartFace, ShelfBacking)> {
    let mut ready = Vec::new();
    // Slow PNGs/archives must not stop button handling. Draw placeholders until ready.
    let deadline = Instant::now() + Duration::from_millis(2);
    while !waiting.is_empty() {
        let Some(left) = deadline.checked_duration_since(Instant::now()) else { break; };
        let Ok(result) = built.recv_timeout(left) else { break; };
        waiting.retain(|key| *key != result.0);
        ready.push(result);
    }
    ready
}

impl Drop for CartFaces {
    fn drop(&mut self) {
        let (lock, wake) = &*self.queue;
        lock.lock().unwrap_or_else(|e| e.into_inner()).closed = true;
        wake.notify_all();
    }
}

fn work(queue: &Shared, outbox: &Sender<(Key, CartFace, ShelfBacking)>, root: &Path) {
    let (lock, wake) = &**queue;
    loop {
        let (key, mut cart) = {
            let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if q.closed {
                    return;
                }
                if let Some(job) = q.jobs.pop_front() {
                    break job;
                }
                q = wake.wait(q).unwrap_or_else(|e| e.into_inner());
            }
        };
        let face = shelf_face(root, &mut cart);
        let backing = ShelfBacking::read(&cart);
        if outbox.send((key, face, backing)).is_err() {
            return;
        }
    }
}

/// A shelf cart's face. The library scan leaves ROM headers unread (1.6.0), and a GBA
/// cart's plastic comes from its game code, so the header is read here, off the UI thread.
pub fn shelf_face(root: &Path, cart: &mut Cart) -> CartFace {
    if cart.platform == slot_store::Platform::Pico8 {
        return slot_ui::cart_face(cart);
    }
    cart.read_header();
    cart_face_with(cart, label_art(root, cart))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stalled_cart_worker_cannot_hold_the_ui_until_its_io_finishes() {
        let (send, recv) = mpsc::channel();
        send.send(((0, 0, 9), CartFace { rgba: Vec::new(), w: 0, h: 0 }, ShelfBacking::Gba)).unwrap();
        let began = Instant::now();
        let ready = visible_ready(&recv, vec![(0, 0, 0), (0, 0, 1)]);
        assert_eq!(ready.len(), 1, "finished off-screen artwork should still be kept");
        assert!(began.elapsed() < Duration::from_millis(100), "visible jobs blocked input");
        // Sender stays alive, representing workers stalled in SD-card reads.
        drop(send);
    }
}
