use std::io::Cursor;

use image::io::Reader;

use crate::kernel::{Kernel, KernelVal};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Pixel(pub u8, pub u8, pub u8, pub u8);

const WHITE: Pixel = Pixel(255, 255, 255, 255);
const BLACK: Pixel = Pixel(0, 0, 0, 255);

impl Pixel {
    pub fn min(&self, other: Pixel) -> Pixel {
        Pixel(
            self.0.min(other.0),
            self.1.min(other.1),
            self.2.min(other.2),
            self.3.min(other.3),
        )
    }

    pub fn max(&self, other: Pixel) -> Pixel {
        Pixel(
            self.0.max(other.0),
            self.1.max(other.1),
            self.2.max(other.2),
            self.3.max(other.3),
        )
    }
}

#[derive(Clone)]
pub struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Image {
    pub fn new_with_data(data: Vec<u8>) -> Self {
        let reader = Reader::new(Cursor::new(&data[..]))
            .with_guessed_format()
            .expect("Couldn't guess file format.");

        let image = reader.decode().expect("Unable to decode image.");

        Self {
            data: image.to_rgba8().to_vec(),
            width: image.width(),
            height: image.height(),
        }
        .binarize(128)
    }

    pub fn get_bitmap_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
        let index = (y as usize * self.width as usize + x as usize) * 4;

        Pixel(
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
            self.data[index + 3],
        )
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: Pixel) {
        let index = (y as usize * self.width as usize + x as usize) * 4;

        self.data[index] = pixel.0;
        self.data[index + 1] = pixel.1;
        self.data[index + 2] = pixel.2;
        self.data[index + 3] = pixel.3;
    }

    pub fn binarize(&self, threshold: u8) -> Image {
        let mut result = self.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.get_pixel(x, y);
                let brightness =
                    pixel.0 as f32 * 0.2126 + pixel.1 as f32 * 0.7152 + pixel.2 as f32 * 0.0722;

                if brightness < threshold.into() {
                    result.set_pixel(x, y, BLACK);
                } else {
                    result.set_pixel(x, y, WHITE);
                }
            }
        }

        result
    }

    pub fn dilate(&self, kernel: Kernel) -> Image {
        self.dilate_or_erode(kernel, false)
    }

    pub fn erode(&self, kernel: Kernel) -> Image {
        self.dilate_or_erode(kernel, true)
    }

    pub fn open(&self, kernel: Kernel) -> Image {
        let mut okernel = Kernel::new();
        okernel.change_dimension(kernel.get_dimension()).unwrap();
        self.erode(okernel.clone()).dilate(okernel)
    }

    pub fn close(&self, kernel: Kernel) -> Image {
        let mut okernel = Kernel::new();
        okernel.change_dimension(kernel.get_dimension()).unwrap();
        self.dilate(okernel.clone()).erode(okernel)
    }

    pub fn hit_or_miss(&self, kernel: Kernel) -> Image {
        let mut result = self.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                result.set_pixel(x, y, self.match_kernel(x, y, &kernel));
            }
        }

        result
    }

    pub fn thinning(&self, kernel: Kernel) -> Self {
        let mut result = self.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let matches = self.match_kernel(x, y, &kernel);
                if matches == WHITE {
                    result.set_pixel(x, y, BLACK);
                }
            }
        }

        result
    }

    pub fn thickening(&self, kernel: Kernel) -> Self {
        let mut result = self.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let matches = self.match_kernel(x, y, &kernel);
                if matches == WHITE {
                    result.set_pixel(x, y, WHITE);
                }
            }
        }

        result
    }

    fn dilate_or_erode(&self, kernel: Kernel, erode: bool) -> Self {
        let mut new_image = self.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.get_min_or_max(x, y, &kernel, erode);
                new_image.set_pixel(x, y, pixel);
            }
        }

        new_image
    }

    fn get_min_or_max(&self, row: u32, col: u32, kernel: &Kernel, erode: bool) -> Pixel {
        let kernel_dim = kernel.get_dimension();
        let kernel_center = (kernel_dim as i32 - 1) / 2;
        let center = self.get_pixel(row, col);

        let mut min = WHITE;
        let mut max = BLACK;

        for y in 0..kernel_dim {
            for x in 0..kernel_dim {
                let image_y = (col + y) as i32 - kernel_center;
                let image_x = (row + x) as i32 - kernel_center;
                if image_y < 0 || image_y >= self.height as i32 {
                    min = min.min(center);
                    max = max.max(center);
                    continue;
                }

                if image_x < 0 || image_x >= self.width as i32 {
                    min = min.min(center);
                    max = max.max(center);
                    continue;
                }

                let pixel = self.get_pixel(image_x as u32, image_y as u32);

                if kernel.get(x, y) != KernelVal::One {
                    continue;
                }

                min = min.min(pixel);
                max = max.max(pixel);
            }
        }

        if erode {
            min
        } else {
            max
        }
    }

    fn match_kernel(&self, row: u32, col: u32, kernel: &Kernel) -> Pixel {
        let kernel_dim = kernel.get_dimension();
        let kernel_center = (kernel_dim as i32 - 1) / 2;
        let center = self.get_pixel(row, col);

        for y in 0..kernel_dim {
            for x in 0..kernel_dim {
                let image_y = (col + y) as i32 - kernel_center;
                let image_x = (row + x) as i32 - kernel_center;
                if image_y < 0 || image_y >= self.height as i32 {
                    if kernel.get(x, y) == KernelVal::Zero {
                        if center != BLACK {
                            return BLACK;
                        }
                    } else if kernel.get(x, y) == KernelVal::One {
                        if center != WHITE {
                            return BLACK;
                        }
                    }

                    continue;
                }

                if image_x < 0 || image_x >= self.width as i32 {
                    if kernel.get(x, y) == KernelVal::Zero {
                        if center != BLACK {
                            return BLACK;
                        }
                    } else if kernel.get(x, y) == KernelVal::One {
                        if center != WHITE {
                            return BLACK;
                        }
                    }

                    continue;
                }

                let pixel = self.get_pixel(image_x as u32, image_y as u32);

                if kernel.get(x, y) == KernelVal::Zero {
                    if pixel != BLACK {
                        return BLACK;
                    }
                } else if kernel.get(x, y) == KernelVal::One {
                    if pixel != WHITE {
                        return BLACK;
                    }
                }
            }
        }

        WHITE
    }
}
