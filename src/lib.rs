extern crate color_quant;
extern crate delta_e;
extern crate image;
extern crate itertools;
extern crate lab;
#[macro_use]
extern crate quick_error;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::mem;
use std::slice;
use std::os::raw::c_void;

use color_quant::NeuQuant;
use delta_e::DE2000;
use image::FilterType::Gaussian;
use image::{DynamicImage, GenericImage, guess_format, load_from_memory, ImageBuffer, ImageFormat, imageops, Pixel,
            Rgb, Rgba};
use itertools::Itertools;
use lab::Lab;

static MAX_SAMPLE_COUNT: u32 = 1000;
static NQ_SAMPLE_FACTION: i32 = 10;
static NQ_PALETTE_SIZE: usize = 256;
static MIN_BLACK: u8 = 8;
static MAX_WHITE: u8 = 247;
static MIN_DISTANCE_FOR_UNIQUENESS: f32 = 10.0;

extern {
    fn log(s: &str, len: usize);
    fn log_nr(nr: usize);
}

quick_error! {
    #[derive(Debug)]
    pub enum DistilError {
        /// Produced when Distil fails to parse the passed path.
        Io(path: String, err: image::ImageError) {
            display("Distil failed to parse the passed image: {}", err)
        }

        /// Produced when the image passed isn't a JPEG or a PNG.
        UnsupportedFormat {
            display("The passed image isn't a JPEG or a PNG")
        }

        /// Produced when Distil can't find any "interesting" colours in a passed image. Colours
        /// are deemed "interesting" if they fall between RGB(8, 8, 8) and RGB(247, 247, 247).
        Uninteresting {
            display("The passed image does not contain any interesting colours")
        }
    }
}

/// Represents a distilled image.
#[derive(Debug, Clone)]
pub struct Distil {
    /// `colors` contains all of the RGB values the image was distilled down
    /// into organised from most-frequent to least-frequent.
    pub colors: Vec<[u8; 3]>,

    /// `color_count` maps the index of each color in `colors` to the total
    /// number of colors that were distilled down into that same color from a
    /// palette of 256.
    ///
    /// It can be used, for example, to weight a colors importance when
    /// distilling multiple palettes into one.
    pub color_count: BTreeMap<usize, usize>,
}

impl Distil {
    fn new(img: DynamicImage) -> Result<Distil, DistilError> {
        // let scaled_img = scale_img(img);

        match quantize(img) {
            Ok(quantized_img) => {
                let color_count = count_colors_as_lab(quantized_img);
                let palette = remove_similar_colors(color_count);

                Ok(distil_palette(palette))
            }
            Err(err) => {
                unsafe { log(&err.to_string(), err.to_string().len()); }

                return Err(err)
            },
        }
    }
}

fn get_image_format(path: &Path) -> Result<ImageFormat, DistilError> {
    if let Ok(mut file) = File::open(path) {
        let mut file_buffer = [0; 16];
        let _ = file.read(&mut file_buffer);

        if let Ok(format) = guess_format(&file_buffer) {
            return Ok(format);
        }
    }

    Err(DistilError::UnsupportedFormat)
}

fn is_supported_format(format: ImageFormat) -> Result<(), DistilError> {
    match format {
        ImageFormat::PNG | ImageFormat::JPEG => {
            return Ok(());
        }
        _ => {
            return Err(DistilError::UnsupportedFormat);
        }
    }
}

/// Proportionally scales the passed image to a size where its total number of
/// pixels does not exceed the value of `MAX_SAMPLE_COUNT`.
fn scale_img(mut img: DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    if width * height > MAX_SAMPLE_COUNT {
        let (width, height) = (width as f32, height as f32);
        let ratio = width / height;

        let scaled_width = (ratio * (MAX_SAMPLE_COUNT as f32)).sqrt() as u32;

        img = img.resize(scaled_width, height as u32, Gaussian);
    }

    img
}

/// Uses the NeuQuant quantization algorithm to reduce the passed image to a
/// palette of `NQ_PALETTE_SIZE` colors.
///
/// Note: NeuQuant is designed to produce images with between 64 and 256
/// colors. As such, `NQ_PALETTE_SIZE`'s value should be kept within those
/// bounds.
fn quantize(img: DynamicImage) -> Result<Vec<Rgb<u8>>, DistilError> {
    match get_pixels(img) {
        Ok(pixels) => {
            let quantized = NeuQuant::new(NQ_SAMPLE_FACTION, NQ_PALETTE_SIZE, &pixels);

            Ok(quantized.color_map_rgb()
                .iter()
                .chunks(3)
                .into_iter()
                .map(|rgb_iter| {
                    let rgb_slice: Vec<u8> = rgb_iter.cloned().collect();
                    Rgb::from_slice(&rgb_slice).clone()
                })
                .collect())
        }
        Err(err) => {
            unsafe { log(&err.to_string(), err.to_string().len()); }
            
            Err(err)
        },
    }
}

/// Processes each of the pixels in the passed image, filtering out any that are
/// transparent or too light / dark to be interesting, then returns a `Vec` of the
/// `Rgba` channels of "interesting" pixels which is intended to be fed into
/// `NeuQuant`.
fn get_pixels(img: DynamicImage) -> Result<Vec<u8>, DistilError> {
    let mut pixels = Vec::new();

    for (_, _, px) in img.pixels() {
        let rgba = px.to_rgba();

        if has_transparency(&rgba) || is_black(&rgba) || is_white(&rgba) {
            continue;
        }

        for channel in px.channels() {
            pixels.push(*channel);
        }
    }

    if pixels.len() == 0 {
        return Err(DistilError::Uninteresting);
    }

    Ok(pixels)
}

/// Checks if the passed pixel is opaque or not.
fn has_transparency(rgba: &Rgba<u8>) -> bool {
    let alpha_channel = rgba[3];

    alpha_channel != 255
}

/// Checks if the passed pixel is too dark to be interesting.
fn is_black(rgba: &Rgba<u8>) -> bool {
    rgba[0] < MIN_BLACK && rgba[1] < MIN_BLACK && rgba[2] < MIN_BLACK
}

/// Checks if the passed pixel is too light to be interesting.
fn is_white(rgba: &Rgba<u8>) -> bool {
    rgba[0] > MAX_WHITE && rgba[1] > MAX_WHITE && rgba[2] > MAX_WHITE
}

/// Maps each unique Lab color in the passed `Vec` of pixels to the total
/// number of times that color appears in the `Vec`.
fn count_colors_as_lab(pixels: Vec<Rgb<u8>>) -> Vec<(Lab, usize)> {
    let color_count_map = pixels.iter()
        .fold(BTreeMap::new(), |mut acc, px| {
            *acc.entry(px.channels()).or_insert(0) += 1;
            acc
        });

    let mut color_count_vec = color_count_map.iter()
        .fold(Vec::new(), |mut acc, (color, count)| {
            let rgb = Rgb::from_slice(&color).to_owned();
            acc.push((Lab::from_rgb(&[rgb[0], rgb[1], rgb[2]]), *count as usize));
            acc
        });

    color_count_vec.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

    color_count_vec
}

fn remove_similar_colors(palette: Vec<(Lab, usize)>) -> Vec<(Lab, usize)> {
    let mut similars = Vec::new();
    let mut refined_palette: Vec<(Lab, usize)> = Vec::new();

    for &(lab_x, count_x) in palette.iter() {
        let mut is_similar = false;

        for (i, &(lab_y, _)) in refined_palette.iter().enumerate() {
            let delta = DE2000::new(lab_x.into(), lab_y.into());

            if delta < MIN_DISTANCE_FOR_UNIQUENESS {
                similars.push((i, lab_x, count_x));
                is_similar = true;
                break;
            }
        }

        if !is_similar {
            refined_palette.push((lab_x, count_x));
        }
    }

    for &(i, lab_y, count) in &similars {
        let lab_x = refined_palette[i].0;
        let (lx, ax, bx) = (lab_x.l, lab_x.a, lab_x.b);
        let (ly, ay, by) = (lab_y.l, lab_y.a, lab_y.b);

        let count_x = refined_palette[i].1 as f32;
        let count_y = count as f32;

        let balanced_l = (lx * count_x + ly * count_y) / (count_x + count_y);
        let balanced_a = (ax * count_x + ay * count_y) / (count_x + count_y);
        let balanced_b = (bx * count_x + by * count_y) / (count_x + count_y);

        refined_palette[i].0 = Lab {
            l: balanced_l,
            a: balanced_a,
            b: balanced_b,
        };

        refined_palette[i].1 += count_y as usize;
    }

    refined_palette.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

    refined_palette
}

/// Organises the produced color palette into something that's useful for a
/// user.
fn distil_palette(palette: Vec<(Lab, usize)>) -> Distil {
    let mut colors = Vec::new();
    let mut color_count = BTreeMap::new();

    for (i, &(lab_color, count)) in palette.iter().enumerate() {
        colors.push(lab_color.to_rgb());
        color_count.insert(i, count);
    }

    Distil {
        colors: colors,
        color_count: color_count,
    }
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
	let mut buf = Vec::with_capacity(size);
	let ptr = buf.as_mut_ptr();
	mem::forget(buf);

	return ptr as *mut c_void;
}

fn process_img(img: DynamicImage, palette_size: usize) -> *mut i32 {
    let err = Box::new([500]);
    match Distil::new(img) {
        Ok(distil) => {
            let mut colors = Vec::new();
            
            for (i, color) in distil.colors.iter().enumerate() {

                colors.push(color);

                if i == palette_size as usize -1 {
                    break;
                }
            }

            let res = Box::new(colors);
            unsafe { log_nr(distil.colors.len()); }

            return Box::into_raw(res) as *mut i32;
        },
        Err(err) => {
            let err_msg: String = err.to_string().to_owned();
            let mut ns: String = "[process_img] ".to_owned();

            ns.push_str(&err_msg);

            unsafe { log(&ns, ns.len()); }

            err.to_string().as_ptr() as *mut i32
        }
    }
}

#[no_mangle]
pub extern "C" fn read_img(buff_ptr: *mut u8, buff_len: usize, palette_size: usize) -> *mut i32 {
	let mut img: Vec<u8> = unsafe { Vec::from_raw_parts(buff_ptr, buff_len, buff_len) };
    let err = Box::new([500]);

    return match image::load_from_memory(&img) {
        Ok(img) => process_img(img, palette_size),
        Err(err) => {
            let err_msg: String = err.to_string().to_owned();
            let mut ns: String = "[load_from_memory] ".to_owned();

            ns.push_str(&err_msg);

            unsafe { log(&ns, ns.len()); }

            err.to_string().as_ptr() as *mut i32
        }
    }
}

fn main() {
	println!("Hello from rust 2");
}