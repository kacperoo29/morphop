use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KernelVal {
    One,
    Zero,
    DontCare,
}

#[derive(Debug, Clone)]
pub struct Kernel {
    dimension: u32,
    data: Vec<KernelVal>,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            dimension: 1,
            data: vec![KernelVal::One],
        }
    }

    pub fn change_dimension(&mut self, dimension: u32) -> Result<(), Box<dyn Error>> {
        if dimension % 2 == 0 {
            return Err("Dimension must be odd.".into());
        }

        self.dimension = dimension;
        self.data = vec![KernelVal::One; (dimension * dimension) as usize];

        Ok(())
    }

    pub fn get_dimension(&self) -> u32 {
        self.dimension
    }

    pub fn set(&mut self, x: u32, y: u32, val: KernelVal) {
        let index = (y as usize * self.dimension as usize + x as usize) as usize;
        self.data[index] = val;
    }

    pub fn get(&self, x: u32, y: u32) -> KernelVal {
        let index = (y as usize * self.dimension as usize + x as usize) as usize;
        self.data[index]
    }
}


