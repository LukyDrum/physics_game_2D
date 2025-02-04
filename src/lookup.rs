use std::collections::LinkedList;

use macroquad::math::Vec2;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{linked_linked_list::LinkedLinkedList, simulation::Particle};

#[derive(Clone)]
pub struct Cell(pub LinkedList<usize>);

impl Cell {
    pub fn empty() -> Self {
        Cell(LinkedList::new())
    }

    pub fn clear(&mut self) {
        self.0 = LinkedList::new();
    }

    pub fn insert(&mut self, index: usize) {
        self.0.push_back(index);
    }
}

pub struct LookUp {
    pub cells: Vec<Vec<Cell>>,
    pub width: f32,
    pub height: f32,
    pub cell_size: f32,
}

impl LookUp {
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

    pub fn insert(&mut self, particle: &Particle, index: usize) {
        let pos = particle.predicted_position;
        if pos.x < 0.0 || pos.x > self.width || pos.y < 0.0 || pos.y > self.height {
            return;
        }

        let col = (pos.x / self.cell_size) as usize;
        let row = (pos.y / self.cell_size) as usize;

        self.cells[row][col].insert(index);
    }

    pub fn get_immediate_neighbors(&self, position: Vec2) -> LinkedLinkedList<usize> {
        self.get_neighbors_in_radius(position, self.cell_size)
    }

    pub fn get_neighbors_in_radius(&self, position: Vec2, radius: f32) -> LinkedLinkedList<usize> {
        if position.x < 0.0
            || position.x > self.width
            || position.y < 0.0
            || position.y > self.height
        {
            return LinkedLinkedList::new();
        }

        let off = (radius / self.cell_size) as i32;

        let mid_col = (position.x / self.cell_size) as i32;
        let mid_row = (position.y / self.cell_size) as i32;

        let mut neighbors = LinkedLinkedList::new();
        for row in (mid_row - off)..=(mid_row + off) {
            for col in (mid_col - off)..=(mid_col + off) {
                // When doing `mid - 1` the result can be negative (-1), casting that to usize will
                // result in the `usize::MAX`, that should practicly always be out of the range of
                // the Vec.
                // TLDR: Underflow is intended behavior here
                if let Some(Cell(indexes)) = self
                    .cells
                    .get(row as usize)
                    .map(|r| r.get(col as usize))
                    .flatten()
                {
                    neighbors.push_back(indexes);
                }
            }
        }

        neighbors
    }
}
