use std::io::Cursor;

use image::io::Reader;

#[derive(Clone, Copy)]
pub struct Pixel(pub u8, pub u8, pub u8, pub u8);

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

        // log if out of bounds
        if index >= self.data.len() {
            log::error!("Pixel out of bounds: {}, {}", x, y);
            log::info!("Image size: {}, {}", self.width, self.height);
        }

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

    pub fn dilate(&self, radius: u32) -> Image {
        self.dilate_or_erode(radius, false)
    }

    pub fn erode(&self, radius: u32) -> Image {
        self.dilate_or_erode(radius, true)
    }

    pub fn open(&self, radius: u32) -> Image {
        self.erode(radius).dilate(radius)
    }

    pub fn close(&self, radius: u32) -> Image {
        self.dilate(radius).erode(radius)
    }

    pub fn hit_and_miss(&self, radius: u32) -> Image {
        let mut result = self.clone();
        let radius = radius as i32;

        for y in 0..self.height {
            for x in 0..self.width {
                let mut pixel = self.get_pixel(x, y);

                for dy in -radius as i32..=radius as i32 {
                    for dx in -radius as i32..=radius as i32 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let other = self.get_pixel(
                            (x as i32 + dx).max(0).min(self.width as i32 - 1) as u32,
                            (y as i32 + dy).max(0).min(self.height as i32 - 1) as u32,
                        );

                        pixel = pixel.min(other);
                    }
                }

                result.set_pixel(x, y, pixel);
            }
        }

        result
    }

    fn dilate_or_erode(&self, radius: u32, erode: bool) -> Self {
        let mut new_image = self.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let mut max = Pixel(0, 0, 0, 0);
                let mut min = Pixel(255, 255, 255, 255);

                for i in 0..radius * 2 + 1 {
                    for j in 0..radius * 2 + 1 {
                        let pixel = self.get_pixel(
                            (x as i32 - radius as i32 + i as i32)
                                .max(0)
                                .min(self.width as i32 - 1) as u32,
                            (y as i32 - radius as i32 + j as i32)
                                .max(0)
                                .min(self.height as i32 - 1) as u32,
                        );

                        max = max.max(pixel);
                        min = min.min(pixel);
                    }
                }

                let val = if erode { min } else { max };

                new_image.set_pixel(x, y, val);
            }
        }

        new_image
    }
}
