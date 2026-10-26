//! Used by the other [`EventLoop`][super::EventLoop] implementations to spawn threads for running
//! tasks in the background without blocking the GUI thread.
//!
//! This is essentially a slimmed down version of the `LinuxEventLoop`.

use anymap3::Entry;
use nice_plug_core::nice_trace;
use parking_lot::Mutex;
use std::sync::{Arc, LazyLock, Weak};
use std::thread::{self, JoinHandle};

use super::MainThreadExecutor;
use crate::util::permit_alloc;

/// See the module's documentation. This is a background thread that can be used to run tasks on.
/// The implementation shares a single thread between all of a plugin's instances hosted in the same
/// process.
pub(crate) struct BackgroundThread<T, E> {
    /// The object that actually executes the task `T`. We'll send a weak reference to this to the
    /// worker thread whenever a task needs to be executed. This allows multiple plugin instances to
    /// share the same worker thread.
    executor: Weak<E>,
    /// A thread that act as our worker thread. When [`schedule()`][Self::schedule()] is called,
    /// this thread will be woken up to execute the task on the executor. When the last worker
    /// thread handle gets dropped the thread is shut down.
    worker_thread: Arc<WorkerThread<T, E>>,
}

/// A handle for the singleton worker thread. This lets multiple instances of the same plugin share
/// a worker thread, and when the last instance gets dropped the worker thread gets terminated.
struct WorkerThread<T, E> {
    tasks_sender: crossbeam::channel::Sender<Message<T, E>>,
    /// The thread's join handle. Joined when the WorkerThread is dropped.
    join_handle: Option<JoinHandle<()>>,
}

/// A message for communicating with the worker thread.
enum Message<T, E> {
    /// A new task for the event loop to execute along with the executor that should execute the
    /// task. A reference to the executor is sent alongside because multiple plugin instances may
    /// share the same background thread.
    Task((T, Weak<E>)),
    /// Shut down the worker thread. Send when the last reference to the thread is dropped.
    Shutdown,
}

impl<T, E> BackgroundThread<T, E>
where
    T: Send + 'static,
    E: MainThreadExecutor<T> + 'static,
{
    pub fn get_or_create(executor: Weak<E>) -> Self {
        Self {
            executor,
            // The same worker thread can be shared by multiple instances. Lifecycle management
            // happens through reference counting.
            worker_thread: get_or_create_worker_thread(),
        }
    }

    pub fn schedule(&self, task: T) -> bool {
        // NOTE: This may check the current thread ID, which involves an allocation whenever this
        //       first happens on a new thread because of the way thread local storage works
        permit_alloc(|| {
            self.worker_thread
                .tasks_sender
                .try_send(Message::Task((task, self.executor.clone())))
                .is_ok()
        })
    }
}

// Rust does not allow us to use the `T` and `E` type variable in statics, so this is a
// workaround to have a singleton that also works if for whatever reason there are multiple `T`
// and `E`s in a single process (won't happen with normal plugin usage, but who knows).
static HANDLE_MAP: LazyLock<Mutex<anymap3::Map<dyn std::any::Any + Send>>> =
    LazyLock::new(|| Mutex::new(anymap3::Map::new()));

impl<T: Send + 'static, E: MainThreadExecutor<T> + 'static> WorkerThread<T, E> {
    fn spawn() -> Self {
        let (tasks_sender, tasks_receiver) =
            crossbeam::channel::bounded(super::TASK_QUEUE_CAPACITY);
        let join_handle = thread::Builder::new()
            .name(String::from("bg-worker"))
            .spawn(move || worker_thread(tasks_receiver))
            .expect("Could not spawn background worker thread");

        Self {
            join_handle: Some(join_handle),
            tasks_sender,
        }
    }
}

impl<T, E> Drop for WorkerThread<T, E> {
    fn drop(&mut self) {
        // The thread is shut down and joined when the handle is dropped
        let _ = self.tasks_sender.send(Message::Shutdown);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Either acquire a handle for an existing worker thread or create one if it does not yet exists.
/// This allows multiple plugin instances to share a worker thread. Reference counting happens
/// automatically as part of this function and `WorkerThreadHandle`'s lifecycle.
fn get_or_create_worker_thread<T, E>() -> Arc<WorkerThread<T, E>>
where
    T: Send + 'static,
    E: MainThreadExecutor<T> + 'static,
{
    let mut handle_map = HANDLE_MAP.lock();

    match handle_map.entry::<Weak<WorkerThread<T, E>>>() {
        Entry::Occupied(mut entry) => {
            let weak = entry.get_mut();
            if let Some(arc) = weak.upgrade() {
                arc
            } else {
                let arc = Arc::new(WorkerThread::spawn());
                *weak = Arc::downgrade(&arc);
                arc
            }
        }
        Entry::Vacant(entry) => {
            let arc = Arc::new(WorkerThread::spawn());
            entry.insert(Arc::downgrade(&arc));
            arc
        }
    }
}

/// The worker thread used in [`EventLoop`] that executes incoming tasks on the event loop's
/// executor.
fn worker_thread<T, E>(tasks_receiver: crossbeam::channel::Receiver<Message<T, E>>)
where
    T: Send,
    E: MainThreadExecutor<T> + 'static,
{
    loop {
        match tasks_receiver.recv() {
            Ok(Message::Task((task, executor))) => match executor.upgrade() {
                Some(e) => e.execute(task, true),
                None => {
                    nice_trace!(
                        "Skipping a task whose executor is no longer alive"
                    );
                    continue;
                }
            },
            Ok(Message::Shutdown) => return,
            Err(err) => {
                nice_trace!(
                    "Worker thread got disconnected unexpectedly, shutting down: {}",
                    err
                );
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Executor(AtomicUsize);
    impl MainThreadExecutor<usize> for Executor {
        fn execute(&self, task: usize, _: bool) { self.0.fetch_add(task, Ordering::Relaxed); }
    }

    #[test]
    fn stale_executor_does_not_stop_other_instances() {
        let (sender, receiver) = crossbeam::channel::unbounded();
        let live = Arc::new(Executor(AtomicUsize::new(0)));
        sender.send(Message::Task((100, Weak::<Executor>::new()))).unwrap();
        sender.send(Message::Task((1, Arc::downgrade(&live)))).unwrap();
        sender.send(Message::Shutdown).unwrap();
        worker_thread(receiver);
        assert_eq!(live.0.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn disconnected_worker_teardown_is_safe() {
        let (tasks_sender, receiver) = crossbeam::channel::unbounded::<Message<usize, Executor>>();
        drop(receiver);
        drop(WorkerThread { tasks_sender, join_handle: None });
    }
}
