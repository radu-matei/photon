//! A high-performance image processing library, available for use both natively and on the web.
//!
//! #### Functions
//! 96 functions are available, including:
//! - **Transformations**: Resize, crop, and flip images.
//! - **Image correction**: Hue rotation, sharpening, brightness adjustment, adjusting saturation, lightening/darkening all within various colour spaces.
//! - **Convolutions**: Sobel filters, blurs, Laplace effects, edge detection, etc.,
//! - **Channel manipulation**: Increasing/decreasing RGB channel values, swapping channels, removing channels, etc.
//! - **Monochrome effects**: Duotoning, greyscaling of various forms, thresholding, sepia, averaging RGB values
//! - **Colour manipulation**: Work with the image in various colour spaces such as HSL, LCh, and sRGB, and adjust the colours accordingly.
//! - **Filters**: Over 30 pre-set filters available, incorporating various effects and transformations.
//! - **Text**: Apply text to imagery in artistic ways, or to watermark, etc.,
//! - **Watermarking**: Watermark images in multiple formats.
//! - **Blending**: Blend images together using 10 different techniques, change image backgrounds.
//!
//! ## Example
//! ```no_run
//! extern crate photon_rs;
//!
//! use photon_rs::channels::alter_red_channel;
//! use photon_rs::native::{open_image};
//!
//! fn main() {
//!     // Open the image (a PhotonImage is returned)
//!     let mut img = open_image("img.jpg").expect("File should open");
//!     // Apply a filter to the pixels
//!     alter_red_channel(&mut img, 25_i16);
//! }
//! ```
//!
//! This crate contains built-in preset functions, which provide default image processing functionality, as well as functions
//! that allow for direct, low-level access to channel manipulation.
//! To view a full demo of filtered imagery, visit the [official website](https://silvia-odwyer.github.io/photon).
//!
//! ### WebAssembly Use
//! To allow for universal communication between the core Rust library and WebAssembly, the functions have been generalised to allow for both native and in-browser use.
//! [Check out the official guide](https://silvia-odwyer.github.io/photon/guide/) on how to get started with Photon on the web.
//!
//! ### Live Demo
//! View the [official demo of WASM in action](https://silvia-odwyer.github.io/photon).

use base64::{decode, encode};
use image::DynamicImage::ImageRgba8;
use image::{GenericImage, GenericImageView};
use serde::{Deserialize, Serialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Provides the image's height, width, and contains the image's raw pixels.
/// For use when communicating between JS and WASM, and also natively.

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhotonImage {
    raw_pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl PhotonImage {
    /// Create a new PhotonImage from a Vec of u8s, which represent raw pixels.
    pub fn new(raw_pixels: Vec<u8>, width: u32, height: u32) -> PhotonImage {
        PhotonImage {
            raw_pixels,
            width,
            height,
        }
    }

    /// Create a new PhotonImage from a base64 string.
    pub fn new_from_base64(base64: &str) -> PhotonImage {
        base64_to_image(base64)
    }

    /// Create a new PhotonImage from a byteslice.
    pub fn new_from_byteslice(vec: Vec<u8>) -> PhotonImage {
        let slice = vec.as_slice();

        let img = image::load_from_memory(slice).unwrap();

        let raw_pixels = img.to_rgba8().to_vec();

        PhotonImage {
            raw_pixels,
            width: img.width(),
            height: img.height(),
        }
    }

    // pub fn new_from_buffer(buffer: &Buffer, width: u32, height: u32) -> PhotonImage {
    //     // Convert a Node.js Buffer into a Vec<u8>
    //     let raw_pixels: Vec<u8> = Uint8Array::new_with_byte_offset_and_length(
    //         &buffer.buffer(),
    //         buffer.byte_offset(),
    //         buffer.length(),
    //     ).to_vec();

    //     PhotonImage {
    //         raw_pixels,
    //         width,
    //         height,
    //     }
    // }

    /// Get the width of the PhotonImage.
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Get the PhotonImage's pixels as a Vec of u8s.
    pub fn get_raw_pixels(&self) -> Vec<u8> {
        self.raw_pixels.clone()
    }

    /// Get the height of the PhotonImage.
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Convert the PhotonImage to base64.
    pub fn get_base64(&self) -> String {
        let mut img = helpers::dyn_image_from_raw(self);
        img = ImageRgba8(img.to_rgba8());

        let mut buffer = vec![];
        img.write_to(&mut buffer, image::ImageOutputFormat::Png)
            .unwrap();
        let base64 = encode(&buffer);

        let res_base64 = format!("data:image/png;base64,{}", base64.replace("\r\n", ""));

        res_base64
    }
}

/// RGB color type.

#[derive(Serialize, Deserialize, Debug)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    /// Create a new RGB struct.
    pub fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

    /// Set the Red value.
    pub fn set_red(&mut self, r: u8) {
        self.r = r;
    }

    /// Get the Green value.
    pub fn set_green(&mut self, g: u8) {
        self.g = g;
    }

    /// Set the Blue value.
    pub fn set_blue(&mut self, b: u8) {
        self.b = b;
    }

    /// Get the Red value.
    pub fn get_red(&self) -> u8 {
        self.r
    }

    /// Get the Green value.
    pub fn get_green(&self) -> u8 {
        self.g
    }

    /// Get the Blue value.
    pub fn get_blue(&self) -> u8 {
        self.b
    }
}

impl From<Vec<u8>> for Rgb {
    fn from(vec: Vec<u8>) -> Self {
        if vec.len() != 3 {
            panic!("Vec length must be equal to 3.")
        }
        Rgb::new(vec[0], vec[1], vec[2])
    }
}

/// RGBA color type.

#[derive(Serialize, Deserialize, Debug)]
pub struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba {
    /// Create a new RGBA struct.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r, g, b, a }
    }

    /// Set the Red value.
    pub fn set_red(&mut self, r: u8) {
        self.r = r;
    }

    /// Get the Green value.
    pub fn set_green(&mut self, g: u8) {
        self.g = g;
    }

    /// Set the Blue value.
    pub fn set_blue(&mut self, b: u8) {
        self.b = b;
    }

    /// Set the alpha value.
    pub fn set_alpha(&mut self, a: u8) {
        self.a = a;
    }

    /// Get the Red value.
    pub fn get_red(&self) -> u8 {
        self.r
    }

    /// Get the Green value.
    pub fn get_green(&self) -> u8 {
        self.g
    }

    /// Get the Blue value.
    pub fn get_blue(&self) -> u8 {
        self.b
    }

    /// Get the alpha value for this color.
    pub fn get_alpha(&self) -> u8 {
        self.a
    }
}

impl From<Vec<u8>> for Rgba {
    fn from(vec: Vec<u8>) -> Self {
        if vec.len() != 4 {
            panic!("Vec length must be equal to 4.")
        }
        Rgba::new(vec[0], vec[1], vec[2], vec[3])
    }
}

/// Convert a base64 string to a PhotonImage.

pub fn base64_to_image(base64: &str) -> PhotonImage {
    let base64_to_vec: Vec<u8> = base64_to_vec(base64);

    let slice = base64_to_vec.as_slice();

    let mut img = image::load_from_memory(slice).unwrap();
    img = ImageRgba8(img.to_rgba8());
    let raw_pixels = img.to_bytes();

    PhotonImage {
        raw_pixels,
        width: img.width(),
        height: img.height(),
    }
}

/// Convert a base64 string to a Vec of u8s.

pub fn base64_to_vec(base64: &str) -> Vec<u8> {
    decode(base64).unwrap()
}

pub mod channels;
pub mod colour_spaces;
pub mod conv;
pub mod effects;
pub mod filters;
pub mod helpers;
mod iter;
pub mod monochrome;
pub mod multiple;
pub mod native;
pub mod noise;
mod tests;
pub mod text;
pub mod transform;
