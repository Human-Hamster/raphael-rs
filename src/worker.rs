use flume::{Sender, Receiver};
use simulator::{Action, Settings};
use simulator::state::InProgress;
use crate::app::SolverEvent;

type Input = (Settings, bool);
type Output = SolverEvent;

pub struct Bridge<U> {
    pub rx: Receiver<U>
}

impl Bridge<Output> {
    pub fn spawn_worker(input: Input) -> Bridge<Output> {
        let (tx, rx) = flume::unbounded::<Output>();

        let worker = Worker::new(input, tx);
        std::thread::spawn(move ||{
            worker.start();
        });

        Bridge::<Output>{rx}
    }
}

pub struct Worker<T, U> {
    input: T,
    tx: Sender<U>
}

impl Worker<Input, Output> {
    fn new(input: Input, tx: Sender<Output>) -> Worker<Input, Output> {
        Worker {
           input, tx
        }
    }
    fn start(&self) {
        let settings = self.input.0;
        let backload_progress = self.input.1;

        let solution_callback = move |actions: &[Action]| {
            self.tx.clone().send(SolverEvent::IntermediateSolution(actions.to_vec())).unwrap();
        };
        let progress_callback = move |progress: f32| {
            self.tx.clone().send(SolverEvent::Progress(progress)).unwrap();
        };

        let final_solution = solvers::MacroSolver::new(
            settings,
            Box::new(solution_callback),
            Box::new(progress_callback),
        )
            .solve(InProgress::new(&settings), backload_progress);
        match final_solution {
            Some(actions) => {
                self.tx.send(SolverEvent::FinalSolution(actions)).unwrap();
            }
            None => {
                self.tx.send(SolverEvent::FinalSolution(Vec::new())).unwrap();
            }
        }
    }
}
