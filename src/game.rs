extern crate rand;
use crate::thread_pool::ThreadPool;
use crate::world::{CellState, ConstructableWorld, World};
use std::marker::{Send, Sync};
use std::sync::{mpsc, Arc};

use rand::{thread_rng, Rng};
use std::ops::Deref;

struct WorldChunk<T> {
    index: u32,
    world: T,
}

pub struct Game<T>
where
    T: World + Clone + ConstructableWorld + 'static,
{
    generation: u32,
    world: Arc<Box<T>>,
    thread_pool: ThreadPool,
    frame_time: f64,
    waiting: f64,
    num_threads: usize,
}

impl<T> Game<T>
where
    T: World + Clone + ConstructableWorld + Send + Sync + 'static,
{
    pub fn new(world: T, fps: i64, num_threads: usize) -> Game<T> {
        let num_threads_sqrt = (num_threads as f64).sqrt().floor() as usize;
        let num_threads = num_threads_sqrt *  num_threads_sqrt;

        Game {
            num_threads,
            generation: 0,
            waiting: 0.0,
            frame_time: 1000.0 / fps as f64,
            thread_pool: ThreadPool::new(num_threads),
            world: Arc::new(Box::new(world.clone())),
        }
    }

    pub fn draw(&self, live_color: [u8; 4], dead_color: [u8; 4], screen: &mut [u8])  {
        for ((_, _, cell), pix) in self.world.into_iterator().zip(screen.chunks_exact_mut(4)) {
            if let CellState::Alive = cell {
                pix.copy_from_slice(&live_color);
            } else {
                pix.copy_from_slice(&dead_color);
            }
        }
    }

    fn get_cell_next_state(world: &T, x: u32, y: u32) -> CellState {
        let x = x as i64;
        let y = y as i64;

        let cell = world
            .get_cell_state(x as u32, y as u32)
            .expect("Updating cell out of bounds");
        let neighbours: [&CellState; 8] = [
            world.get_cell_state_wrapped(x - 1, y - 1),
            world.get_cell_state_wrapped(x - 1, y),
            world.get_cell_state_wrapped(x - 1, y + 1),
            world.get_cell_state_wrapped(x, y + 1),
            world.get_cell_state_wrapped(x + 1, y + 1),
            world.get_cell_state_wrapped(x + 1, y),
            world.get_cell_state_wrapped(x + 1, y - 1),
            world.get_cell_state_wrapped(x, y - 1),
        ];

        let alive_neighbours_count = neighbours.iter().fold(0, |acc, c| {
            if let CellState::Alive = c {
                acc + 1
            } else {
                acc
            }
        });

        match cell {
            CellState::Alive => match alive_neighbours_count {
                2 | 3 => CellState::Alive,
                _ => CellState::Dead,
            },
            CellState::Dead => match alive_neighbours_count {
                3 => CellState::Alive,
                _ => CellState::Dead,
            },
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.waiting += dt * 1000.0;

        // @TODO Calcular en el constructor para no repetir el cálculo
        let split_grid_side_count = (self.num_threads as f64).sqrt().ceil() as u32;

        // println!("Waiting {}, frame_time: {}", self.waiting, self.frame_time);
        while self.waiting >= self.frame_time {

            // println!("Generation {}", self.generation);
            let (sender, receiver) = mpsc::channel();
            let (width, height) = self.world.get_size();

            for i in 0..self.num_threads {
                let index = i as u32;
                let source_world = self.world.clone();
                let sender = sender.clone();
                self.thread_pool.execute(move || {
                    let (x, y, width, height) = Self::get_chunk_limits(
                        width,
                        height,
                        index,
                        split_grid_side_count,
                        split_grid_side_count,
                    );
                    let source_world = source_world.deref();
                    let result = Self::get_chunk_next_state(source_world, x, y, width, height);

                    sender
                        .send(WorldChunk {
                            index,
                            world: result,
                        })
                        .expect("Unable to send result from thread");
                });
            }

            let mut results = Vec::with_capacity(self.num_threads);
            for _ in 0..self.num_threads {
                let result = receiver
                    .recv()
                    .expect("Failed to receive result from thread");

                results.push(result);
            }

            let mut new_world = Box::new(T::new(width, height));
            for WorldChunk { index, world } in results.iter() {
                let (x, y, _, _) =
                    Self::get_chunk_limits(width, height, *index, split_grid_side_count, split_grid_side_count);
                Self::apply_chunk(&mut new_world, world, x, y);
            }

            self.world = Arc::new(new_world);
            self.generation += 1;
            self.waiting -= self.frame_time;
        }
    }

    fn get_chunk_limits(
        width: u32,
        height: u32,
        index: u32,
        num_rows: u32,
        num_cols: u32,
    ) -> (u32, u32, u32, u32) {
        let chunk_width = (width as f64 / num_cols as f64).ceil() as u32;
        let chunk_height = (height as f64 / num_rows as f64).ceil() as u32;
        let x = (index % num_cols) * chunk_width;
        let y = (index / num_rows) * chunk_height;

        (x, y, chunk_width, chunk_height)
    }

    fn get_chunk_next_state(source: &T, x: u32, y: u32, grid_width: u32, grid_height: u32) -> T {
        let (world_width, world_height) = source.get_size();
        let width = std::cmp::min(x + grid_width, world_width) - x;
        let height = std::cmp::min(y + grid_height, world_height) - y;

        let mut output = T::new(width, height);

        for i in 0..width {
            for j in 0..height {
                let next_state = Self::get_cell_next_state(source, i + x, j + y);
                output.set_cell_state(i, j, next_state.clone());
            }
        }
        output
    }

    fn apply_chunk(destination: &mut T, source: &T, x: u32, y: u32) {
        for (cell_x, cell_y, state) in source.into_iterator() {
            destination.set_cell_state(x + cell_x, y + cell_y, state.clone());
        }
    }

    pub fn populate(&mut self, num_cells: u32) {
        let mut rng = thread_rng();
        let (width, height) = self.world.get_size();

        for _ in 0..num_cells {
            let x = rng.gen_range(0, width);
            let y = rng.gen_range(0, height);

            if let Some(w) = Arc::get_mut(&mut self.world) {
                w.set_cell_state(x, y, CellState::Alive);
            }
        }
    }
}
