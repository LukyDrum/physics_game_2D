use std::collections::LinkedList;

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::math::Vector2;
use crate::utility::LinkedLinkedList;

#[derive(Clone)]
struct Cell<T>(pub LinkedList<T>)
where
    T: Clone + Copy + Send;

impl<T> Cell<T>
where
    T: Clone + Copy + Send,
{
    pub fn empty() -> Self {
        Cell(LinkedList::new())
    }

    pub fn clear(&mut self) {
        self.0 = LinkedList::new();
    }

    pub fn insert(&mut self, item: T) {
        self.0.push_back(item);
    }
}

pub struct LookUp<T>
where
    T: Clone + Copy + Send,
{
    cells: Vec<Vec<Cell<T>>>,
    pub width: f32,
    pub height: f32,
    pub cell_size: f32,
}

impl<T> LookUp<T>
where
    T: Clone + Copy + Send,
{
    /// Cell size should be equal to smoothing radius
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let mut cols_count = (width / cell_size) as usize;
        let mut rows_count = (height / cell_size) as usize;
        if width % cell_size > 0.0 {
            cols_count += 1;
        }
        if height % cell_size > 0.0 {
            rows_count += 1;
        }

        LookUp {
            cells: vec![vec![Cell::empty(); cols_count]; rows_count],
            width,
            height,
            cell_size,
        }
    }

    pub fn clear(&mut self) {
        self.cells
            .par_iter_mut()
            .for_each(|row| row.par_iter_mut().for_each(|cell| cell.clear()));
    }

    pub fn insert(&mut self, position: &Vector2<f32>, item: T) {
        let pos = position;
        if pos.x < 0.0 || pos.x > self.width || pos.y < 0.0 || pos.y > self.height {
            return;
        }

        let col = (pos.x / self.cell_size) as usize;
        let row = (pos.y / self.cell_size) as usize;

        if let Some(cell) = self.cells.get_mut(row).and_then(|row| row.get_mut(col)) {
            cell.insert(item);
        }
    }

    pub fn get_immediate_neighbors(&self, position: &Vector2<f32>) -> LinkedLinkedList<T> {
        self.get_neighbors_in_radius(position, self.cell_size)
    }

    pub fn get_neighbors_in_radius(
        &self,
        position: &Vector2<f32>,
        radius: f32,
    ) -> LinkedLinkedList<T> {
        if position.x < 0.0
            || position.x > self.width
            || position.y < 0.0
            || position.y > self.height
        {
            return LinkedLinkedList::default();
        }

        let off = (radius / self.cell_size) as i32;

        let mid_col = (position.x / self.cell_size) as i32;
        let mid_row = (position.y / self.cell_size) as i32;

        let mut neighbors = LinkedLinkedList::default();
        for row in (mid_row - off)..=(mid_row + off) {
            for col in (mid_col - off)..=(mid_col + off) {
                // When doing `mid - 1` the result can be negative (-1), casting that to usize will
                // result in the `usize::MAX`, that should practicly always be out of the range of
                // the Vec.
                // TLDR: Underflow is intended behavior here
                if let Some(Cell(indexes)) = self
                    .cells
                    .get(row as usize)
                    .and_then(|r| r.get(col as usize))
                {
                    neighbors.push_back(indexes);
                }
            }
        }

        neighbors
    }
}
