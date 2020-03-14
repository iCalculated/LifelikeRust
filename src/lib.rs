mod utils;
extern crate js_sys;
extern crate fixedbitset;
use fixedbitset::FixedBitSet;
extern crate web_sys;
use std::fmt;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// toggle for booleans
trait BoolToggle {
    fn toggle(&mut self);
}

impl BoolToggle for bool {
    fn toggle(&mut self) {
        *self = !*self
    }
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    size: usize,
    cells: FixedBitSet,
    first_frame: bool,
}

impl Universe {
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row,col);
            self.cells.set(idx, true); 
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 128;
        let height = 128;
        let size = (width * height) as usize;
        let first_frame = true;
        let cells = FixedBitSet::with_capacity(2 * size);
        Universe { width, height, size, cells, first_frame, }
    }

    pub fn random_fill(&mut self) {
        if self.first_frame {
            for i in 0..self.size {
                self.cells.set(i, js_sys::Math::random()<0.30); 
            }
        } else {
            for i in 0..self.size {
                self.cells.set(self.size+i, js_sys::Math::random()<0.30); 
            }
        }
        self.first_frame.toggle();
    }

    pub fn wipe(&mut self) {
        self.cells.clear();
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        for i in 0..(self.height*width) as usize {
            self.cells.set(i, false); 
        }
        self.size = (width * self.height) as usize;
        self.cells.grow(self.size);
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        for i in 0..(self.width*height) as usize {
            self.cells.set(i, false); 
        }
        self.size = (self.width * height) as usize;
        self.cells.grow(self.size);
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        if self.first_frame {
            self.cells.toggle(self.size + idx); 
        } else {
            self.cells.toggle(idx); 
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn frame(&self) -> bool {
        self.first_frame
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    pub fn tick(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row,col);
                let (lookup_idx, set_idx) = {
                    if self.first_frame {
                        (idx + self.size, idx)
                    } else {
                        (idx, idx + self.size) 
                    }
                };
                let cell = self.cells[lookup_idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                /*
                log!(
                    "cell[{}, {}] is initially {:?} and has {} live neighbors",
                    row,
                    col,
                    cell, 
                    live_neighbors
                );
                */

                self.cells.set(set_idx, match (cell, live_neighbors) {
                    (false, 3) => true,
                    (false, _) => false,
                    (true, 2) | (true, 3) => true,
                    (true, _) => false,
                });
                //log!("  it becomes {:?}", self.cells[set_idx]);
            }
        }
        self.first_frame.toggle();
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;
        //modulo is too expensive...
        /*
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (col + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        */
        let north = if row == 0 {
            self.height - 1
        } else {
            row - 1
        };
        let south = if row == self.height - 1 {
            0
        } else {
            row + 1
        };
        let west = if col == 0 {
            self.width - 1
        } else {
            col - 1
        };
        let east = if col == self.width - 1 {
            0
        } else {
            col + 1
        };

        if !self.first_frame {
            count += self.cells[self.get_index(north,west)] as u8 +
                self.cells[self.get_index(north,east)] as u8 +
                self.cells[self.get_index(south,west)] as u8 +
                self.cells[self.get_index(south,east)] as u8 + 
                self.cells[self.get_index(north,col)] as u8 +
                self.cells[self.get_index(south,col)] as u8 +
                self.cells[self.get_index(row,east)] as u8 +
                self.cells[self.get_index(row,west)] as u8; 
        } else {
            count += self.cells[self.size + self.get_index(north,west)] as u8 +
                self.cells[self.size + self.get_index(north,east)] as u8 +
                self.cells[self.size + self.get_index(south,west)] as u8 +
                self.cells[self.size + self.get_index(south,east)] as u8 + 
                self.cells[self.size + self.get_index(north,col)] as u8 +
                self.cells[self.size + self.get_index(south,col)] as u8 +
                self.cells[self.size + self.get_index(row,east)] as u8 +
                self.cells[self.size + self.get_index(row,west)] as u8; 
        }
            
        count
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..(2*self.height) {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let symbol = if !self.cells[idx] { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f,"\n")?;
        }
        Ok(())
    }
}
