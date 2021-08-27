use std::mem;

pub struct WorldIterator<'a> {
    world: &'a dyn World,
    curr: u32,
}

impl<'a> WorldIterator<'a> {
    pub fn new(world: &'a impl World) -> WorldIterator {
        WorldIterator { world, curr: 0 }
    }
}

impl<'a> Iterator for WorldIterator<'a> {
    type Item = (u32, u32, &'a CellState);

    fn next(&mut self) -> Option<(u32, u32, &'a CellState)> {
        let (width, _) = self.world.get_size();
        let x = self.curr % width;
        let y = self.curr / width;
        self.curr += 1;

        self.world.get_cell_state(x, y).map(|c| (x, y, c))
    }
}

#[derive(Clone, Debug)]
pub enum CellState {
    Alive,
    Dead,
}

pub trait World {
    fn get_size(&self) -> (u32, u32);
    fn get_cell_state_wrapped(&self, x: i64, y: i64) -> &CellState;
    fn get_cell_state(&self, x: u32, y: u32) -> Option<&CellState>;
    fn set_cell_state(&mut self, x: u32, y: u32, state: CellState) -> CellState;
    fn into_iterator(&self) -> WorldIterator;
}

pub trait ConstructableWorld {
    fn new(with: u32, height: u32) -> Self;
}

#[derive(Clone)]
pub struct ArrayWorld {
    world: Vec<CellState>,
    width: u32,
    height: u32,
}

impl ArrayWorld {
    fn to_world_index(&self, x: u32, y: u32) -> u32 {
        (y * self.width) + x
    }

    fn wrap(&self, x: i64, y: i64) -> (u32, u32) {
        let width = self.width as i64;
        let height = self.height as i64;

        let x = ((x % width) + width) % width;
        let y = ((y % height) + height) % height;
        (x as u32, y as u32)
    }
}
impl ConstructableWorld for ArrayWorld {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            world: vec![CellState::Dead; (width * height) as usize],
        }
    }
}

impl World for ArrayWorld {
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn get_cell_state_wrapped(&self, x: i64, y: i64) -> &CellState {
        let (x, y) = self.wrap(x, y);
        self.get_cell_state(x, y).unwrap_or(&CellState::Dead)
    }

    fn get_cell_state(&self, x: u32, y: u32) -> Option<&CellState> {
        self.world.get(self.to_world_index(x, y) as usize)
    }

    fn set_cell_state(&mut self, x: u32, y: u32, state: CellState) -> CellState {
        let index = self.to_world_index(x, y);
        mem::replace(&mut self.world[index as usize], state)
    }

    fn into_iterator(&self) -> WorldIterator {
        WorldIterator::new(self)
    }
}
