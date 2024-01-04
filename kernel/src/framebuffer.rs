use bootloader_api::info::{FrameBuffer, FrameBufferInfo, Optional, PixelFormat};
use core::{
    fmt::{self, Write},
    ptr,
};
use font_constants::BACKUP_CHAR;
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};
use spin::Mutex;

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;
/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

/// Constants for the usage of the [`noto_sans_mono_bitmap`] crate.
mod font_constants {
    use super::*;

    /// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
    /// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    /// The width of each single symbol of the mono space font.
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

    /// Backup character if a desired symbol is not available by the font.
    /// The '�' character requires the feature "unicode-specials".
    pub const BACKUP_CHAR: char = '�';

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

pub static LOGGER: Mutex<Option<FrameBufferWriter>> = Mutex::new(None);

pub fn init_print(framebuffer: &'static mut Optional<FrameBuffer>) {
    let framebuffer = framebuffer.as_mut().unwrap();
    let info = framebuffer.info();
    let mut logger = LOGGER.lock();
    *logger = Some(FrameBufferWriter::new(framebuffer.buffer_mut(), info));
}

/// Allows logging text to a pixel-based framebuffer.
pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl FrameBufferWriter {
    /// Creates a new logger that uses the given framebuffer.
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        logger.clear();
        logger
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    /// Erases all text on the screen. Resets `self.x_pos` and `self.y_pos`.
    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos =
                    self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    fn write_char_color_rgb(&mut self, c: char, color: [u8; 3]) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos =
                    self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }
                self.write_rendered_char_color_rgb(get_char_raster(c), color);
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x_pos`.
    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_rendered_char_color_rgb(&mut self, rendered_char: RasterizedChar, color: [u8; 3]) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel_color_rgb(self.x_pos + x, self.y_pos + y, *byte, color);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn get_scaled_color(color: [u8; 3], intensity: u8) -> [u8; 3] {
        let mut scaled_colors = [0; 3];
        for i in 0..3 {
            let color = color[i] as u16;
            let intensity = intensity as u16;
            let scaled_color = color * intensity;
            let scaled_color = scaled_color / 255;
            scaled_colors[i] = scaled_color as u8;
        }
        scaled_colors
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        self.write_pixel_color_rgb(x, y, intensity, [255, 255, 255])
    }

    fn write_pixel_color_rgb(&mut self, x: usize, y: usize, intensity: u8, color: [u8; 3]) {
        let pixel_offset = y * self.info.stride + x;
        let color = Self::get_scaled_color(color, intensity);
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [color[0], color[1], color[2], 0],
            PixelFormat::Bgr => [color[2], color[1], color[0], 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut locked = LOGGER.lock();
        let locked = locked.as_mut();
        if let Some(logger) = locked {
            logger.write_fmt(args).unwrap();
        }
    });
}

#[doc(hidden)]
pub fn _print_color_rgb(args: fmt::Arguments, color: [u8; 3]) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut locked = LOGGER.lock();
        let locked = locked.as_mut();
        if let Some(logger) = locked {
            if let Some(chars) = args.as_str().map(|c| c.chars()) {
                for c in chars {
                    logger.write_char_color_rgb(c, color);
                }
            }
        }
    });
}

#[macro_export]
macro_rules! print_color {
    ($color:expr, $($arg:tt)*) => ($crate::framebuffer::_print_color_rgb(format_args!($($arg)*), $color));
}

#[macro_export]
macro_rules! println_color {
    ($color:expr, $($arg:tt)*) => ($crate::print_color!($color, "{}\n", format_args!($($arg)*)));
}
