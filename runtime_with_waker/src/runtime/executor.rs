use crate::future::{Future, PollState};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, Thread},
};
/// > Executor's features:
/// - Holds many top-level futures and switches between them
/// - Enables to spawn new top-level futures from anywhere
/// - Hand out `Waker` types so that can sleep when there is nothing to do and wake up when one of the top-level futures can progress
/// - Enables to run several executors by having each run on its dedicated OS thread
///
///

type Task = Box<dyn Future<Output = String>>;

// Allows defining a static variable that's unique to the thread it's first called from
thread_local! {
static CURRENT_EXEC: ExecutorCore = ExecutorCore::default();
}

#[derive(Default)]
struct ExecutorCore {
    /// Хранит все top-level футуры ассоциируемые с Executor-ом в данном потоке
    tasks: RefCell<HashMap<usize, Task>>,
    /// Хранит ID задач, которые должны опрашиваться Executor-ом
    ready_queue: Arc<Mutex<Vec<usize>>>,
    next_id: Cell<usize>,
}

/// ### Spawns a new top-level future
/// 1. Получение следующего доступного ID для задачи
/// 2. Запись задачи в список задач Executor-а с данным ID
/// 3. Добавление ID в очередь задач
/// 4. Увеличение счетчика ID
pub fn spawn<F>(future: F)
where
    F: Future<Output = String> + 'static,
{
    CURRENT_EXEC.with(|e| {
        let id = e.next_id.get();
        e.tasks.borrow_mut().insert(id, Box::new(future));
        e.ready_queue.lock().map(|mut q| q.push(id)).unwrap();
        e.next_id.set(id + 1);
    });
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    /// Блокирует `read_queue` и возвращает ID готовой к прогрессированию задачи с конца очереди
    fn pop_ready(&self) -> Option<usize> {
        CURRENT_EXEC.with(|q| q.ready_queue.lock().map(|mut q| q.pop()).unwrap())
    }
    /// Удаляет задачу с конца очереди с выбранным ID, возвращает ее
    fn get_future(&self, id: usize) -> Option<Task> {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().remove(&id))
    }
    fn get_waker(&self, id: usize) -> Waker {
        Waker {
            id,
            thread: thread::current(),
            ready_queue: CURRENT_EXEC.with(|q| q.ready_queue.clone()),
        }
    }
    /// Запись задачи в список задач Executor-а
    fn insert_task(&self, id: usize, task: Task) {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().insert(id, task));
    }

    /// Возвращает количество задач в очереди
    fn task_count(&self) -> usize {
        CURRENT_EXEC.with(|q| q.tasks.borrow().len())
    }

    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = String> + 'static,
    {
        spawn(future);
        loop {
            while let Some(id) = self.pop_ready() {
                let mut future = match self.get_future(id) {
                    Some(f) => f,
                    // guard against false wakeups
                    None => continue,
                };
                let waker = self.get_waker(id);
                match future.poll(&waker) {
                    PollState::NotReady => self.insert_task(id, future),
                    PollState::Ready(_) => continue,
                }
            }
            let task_count = self.task_count();
            let name = thread::current().name().unwrap_or_default().to_string();
            if task_count > 0 {
                println!("{name}: {task_count} pending tasks. Sleep until notified.");
                thread::park();
            } else {
                println!("{name}: All tasks are finished");
                break;
            }
        }
    }
}

#[derive(Clone)]
pub struct Waker {
    /// Объект текущего потока выполнения (Executor)
    thread: Thread,
    /// ID задачи, с которой связан Waker
    id: usize,
    /// Разделяемая (с Executor) ссылка на очередь готовых задач ()
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        self.ready_queue
            .lock()
            .map(|mut q| q.push(self.id))
            .unwrap();
        // Пробуждения потока в котором выполняется Executor
        self.thread.unpark();
    }
}
