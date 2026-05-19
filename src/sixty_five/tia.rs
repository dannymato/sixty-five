#![allow(dead_code)]

use std::{cell::RefCell, fmt::Display, rc::Rc, time::Instant};

use bytemuck::{Pod, Zeroable};
use enum_display::EnumDisplay;
use macroquad::{
    color::WHITE,
    prelude::vec2,
    texture::{self, draw_texture_ex, DrawTextureParams, Texture2D},
    time::{self, draw_fps},
    window::{self, screen_height, screen_width},
};
use num_derive::FromPrimitive;

use super::memory_bus::{BusRead, BusWrite};

#[repr(u8)]
#[derive(Copy, Clone, Default)]
enum PlayerSize {
    #[default]
    OneCopy = 0,
    TwoCopiesClose = 1,
    TwoCopiesMedium = 2,
    ThreeCopiesClose = 3,
    TwoCopiesWide = 4,
    DoubleSizePlayer = 5,
    ThreeCopiesMedium = 6,
    QuadSizedPlayer = 7,
}

#[derive(Debug, Clone, Default)]
struct Collisions {
    // TODO: Fill this out with the easiest DS
}

#[derive(Clone, Copy, Default, FromPrimitive, EnumDisplay)]
enum Color {
    #[default]
    White = 0x0,
    Gold = 0x1,
    Orange = 0x2,
    BrightOrange = 0x3,
    Pink = 0x4,
    Purple = 0x5,
    PurpleBlue = 0x6,
    Blue = 0x7,
    Blue2 = 0x8,
    LightBlue = 0x9,
    Torquoise = 0xa,
    GreenBlue = 0xb,
    Green = 0xc,
    YellowGreen = 0xd,
    OrangeGreen = 0xe,
    LightOrange = 0xf,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
struct ElementColor(u8, u8, u8, u8);

impl Display for ElementColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = (self.0 as u32) << 16 | (self.1 as u32) << 8 | self.2 as u32;
        write!(f, "{:x}", value)
    }
}

impl From<u8> for ElementColor {
    fn from(value: u8) -> Self {
        match value {
            // These are the NTSC colors, if we want to support PAL or SECAM or B/W we'll have to
            // change
            0x00 | 0x01 => ElementColor(0x00, 0x00, 0x00, 0xFF),
            0x10 | 0x11 => ElementColor(0x44, 0x44, 0x00, 0xFF),
            0x20 | 0x21 => ElementColor(0x70, 0x28, 0x00, 0xFF),
            0x30 | 0x31 => ElementColor(0x84, 0x18, 0x00, 0xFF),
            0x40 | 0x41 => ElementColor(0x88, 0x00, 0x00, 0xFF),
            0x50 | 0x51 => ElementColor(0x78, 0x00, 0x5C, 0xFF),
            0x60 | 0x61 => ElementColor(0x48, 0x00, 0x78, 0xFF),
            0x70 | 0x71 => ElementColor(0x14, 0x00, 0x84, 0xFF),
            0x80 | 0x81 => ElementColor(0x00, 0x00, 0x88, 0xFF),
            0x90 | 0x91 => ElementColor(0x00, 0x18, 0x7C, 0xFF),
            0x02 | 0x03 => ElementColor(0x40, 0x40, 0x40, 0xFF),
            0x12 | 0x13 => ElementColor(0x64, 0x64, 0x10, 0xFF),
            0x22 | 0x23 => ElementColor(0x84, 0x44, 0x14, 0xFF),
            0x32 | 0x33 => ElementColor(0x98, 0x34, 0x18, 0xFF),
            0x42 | 0x43 => ElementColor(0x9C, 0x20, 0x20, 0xFF),
            0x52 | 0x53 => ElementColor(0x8C, 0x20, 0x74, 0xFF),
            0x62 | 0x63 => ElementColor(0x60, 0x20, 0x90, 0xFF),
            0x72 | 0x73 => ElementColor(0x30, 0x20, 0x98, 0xFF),
            0x82 | 0x83 => ElementColor(0x1C, 0x20, 0x9C, 0xFF),
            0x92 | 0x93 => ElementColor(0x1C, 0x38, 0x90, 0xFF),
            0x04 | 0x05 => ElementColor(0x6C, 0x6C, 0x6C, 0xFF),
            0x14 | 0x15 => ElementColor(0x84, 0x84, 0x24, 0xFF),
            0x24 | 0x25 => ElementColor(0x98, 0x5C, 0x28, 0xFF),
            0x34 | 0x35 => ElementColor(0xAC, 0x50, 0x30, 0xFF),
            0x44 | 0x45 => ElementColor(0xB0, 0x3C, 0x3C, 0xFF),
            0x54 | 0x55 => ElementColor(0xA0, 0x3C, 0x88, 0xFF),
            0x64 | 0x65 => ElementColor(0x78, 0x3C, 0xA4, 0xFF),
            0x74 | 0x75 => ElementColor(0x4C, 0x3C, 0xAC, 0xFF),
            0x84 | 0x85 => ElementColor(0x38, 0x40, 0xB0, 0xFF),
            0x94 | 0x95 => ElementColor(0x38, 0x54, 0xA8, 0xFF),
            0x06 | 0x07 => ElementColor(0x90, 0x90, 0x90, 0xFF),
            0x16 | 0x17 => ElementColor(0xA0, 0xA0, 0x34, 0xFF),
            0x26 | 0x27 => ElementColor(0xAC, 0x78, 0x3C, 0xFF),
            0x36 | 0x37 => ElementColor(0xC0, 0x68, 0x48, 0xFF),
            0x46 | 0x47 => ElementColor(0xC0, 0x58, 0x58, 0xFF),
            0x56 | 0x57 => ElementColor(0xB0, 0x58, 0x9C, 0xFF),
            0x66 | 0x67 => ElementColor(0x8C, 0x58, 0xB8, 0xFF),
            0x76 | 0x77 => ElementColor(0x68, 0x58, 0xC0, 0xFF),
            0x86 | 0x87 => ElementColor(0x50, 0x5C, 0xC0, 0xFF),
            0x96 | 0x97 => ElementColor(0x50, 0x70, 0xBC, 0xFF),
            0x08 | 0x09 => ElementColor(0xB0, 0xB0, 0xB0, 0xFF),
            0x18 | 0x19 => ElementColor(0xB8, 0xB8, 0x40, 0xFF),
            0x28 | 0x29 => ElementColor(0xBC, 0x8C, 0x4C, 0xFF),
            0x38 | 0x39 => ElementColor(0xD0, 0x80, 0x5C, 0xFF),
            0x48 | 0x49 => ElementColor(0xD0, 0x70, 0x70, 0xFF),
            0x58 | 0x59 => ElementColor(0xC0, 0x70, 0xB0, 0xFF),
            0x68 | 0x69 => ElementColor(0xA0, 0x70, 0xCC, 0xFF),
            0x78 | 0x79 => ElementColor(0x7C, 0x70, 0xD0, 0xFF),
            0x88 | 0x89 => ElementColor(0x68, 0x74, 0xD0, 0xFF),
            0x98 | 0x99 => ElementColor(0x68, 0x88, 0xCC, 0xFF),
            _ => panic!("somehow something bad happened"),
        }
    }
}

#[derive(Clone, Default)]
struct TIAState {
    vblank: bool,
    vsync: bool,
    playfield_reflection: bool,
    plafield_color: bool,
    ball_above_missles: bool,
    ball_size: u8,
    playfield: [bool; 20],
    missile_0_size: u8,
    missile_1_size: u8,
    player_0_size: PlayerSize,
    player_1_size: PlayerSize,
    player_0: u8,
    player_1: u8,
    missile_0: bool,
    missile_1: bool,
    ball: bool,
    reflect_player_0: bool,
    reflect_player_1: bool,
    player_0_motion: i8,
    player_1_motion: i8,
    missile_0_motion: i8,
    missile_1_motion: i8,
    ball_motion: i8,
    collisions: Collisions,
    up0_color: ElementColor,
    up1_color: ElementColor,
    pf_color: ElementColor,
    bk_color: ElementColor,
}

impl TIAState {
    fn output_pixel(&self, _pixel_clock: u32) -> ElementColor {
        if self.vblank {
            return ElementColor(0, 0, 0, 0xff);
        }
        self.bk_color
    }
}

pub struct WrappedTIA(Rc<RefCell<Tia>>);

impl WrappedTIA {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Tia::new())))
    }

    pub fn read_byte(&self, addr: super::data_types::Word) -> super::data_types::Byte {
        self.0.borrow().read_byte(addr)
    }

    pub fn write_byte(&self, addr: super::data_types::Word, data: super::data_types::Byte) {
        self.0.borrow_mut().write_byte(addr, data)
    }

    pub async fn tick_clock(&self, clock: u32) {
        self.0.borrow_mut().tick_clock(clock).await;
    }
}

// Includes VSYNC and VBLANK since those can be variable
const SCANLINE_COUNT: usize = 262;
const HOR_CLOCK_COUNT: usize = 160;
const HORIZONTAL_BLANK: u32 = 68;

const PIXEL_COUNT: usize = SCANLINE_COUNT * HOR_CLOCK_COUNT;

const CPU_CLOCK_COUNT: u32 = PIXEL_COUNT as u32 * 3;

pub struct Tia {
    current_state: TIAState,
    current_clock_count: u32,
    current_framebuffer: [ElementColor; PIXEL_COUNT],
    render_next_tick: bool,
    texture: Texture2D,
}

impl Tia {
    pub fn new() -> Self {
        let framebuffer = [ElementColor::default(); PIXEL_COUNT];

        let texture = Texture2D::from_rgba8(
            HOR_CLOCK_COUNT as u16,
            SCANLINE_COUNT as u16,
            bytemuck::cast_slice(&framebuffer),
        );
        texture.set_filter(texture::FilterMode::Nearest);

        Self {
            current_state: Default::default(),
            current_clock_count: 0,
            current_framebuffer: framebuffer,
            render_next_tick: false,
            texture,
        }
    }

    pub async fn tick_clock(&mut self, clocks: u32) {
        if self.render_next_tick || self.current_clock_count >= CPU_CLOCK_COUNT {
            self.render_frame().await;
        }
        for _i in 0..clocks {
            self.current_clock_count += 1;

            if self.current_clock_count % 3 == 0 {
                let pixel_idx = self.current_clock_count / 3;
                self.current_framebuffer[pixel_idx as usize] =
                    self.current_state.output_pixel(pixel_idx);
            }
        }
    }

    fn horizontal_pos(&self) -> u32 {
        self.current_clock_count % HOR_CLOCK_COUNT as u32
    }

    fn is_hblank(&self) -> bool {
        self.horizontal_pos() < HORIZONTAL_BLANK
    }

    fn is_vblank(&self) -> bool {
        self.current_state.vblank || self.current_state.vsync
    }

    async fn render_frame(&mut self) {
        self.texture.update_from_bytes(
            HOR_CLOCK_COUNT as u32,
            SCANLINE_COUNT as u32,
            bytemuck::cast_slice(&self.current_framebuffer),
        );

        draw_texture_ex(
            &self.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        draw_fps();

        let start = Instant::now();
        window::next_frame().await;

        println!("Next frame took {:?}", start.elapsed());
        println!("Time since last frame {}", time::get_frame_time());

        for pixel in self.current_framebuffer.iter_mut() {
            *pixel = ElementColor(0, 0, 0, 0xff);
        }
        self.current_clock_count = 0;
        self.render_next_tick = false;
    }
}

impl BusRead for Tia {
    fn read_byte(&self, addr: super::data_types::Word) -> super::data_types::Byte {
        let lower_bytes = addr & 0x00FF;
        match lower_bytes {
            _ => 0,
        }
    }
}

impl BusWrite for Tia {
    fn write_byte(&mut self, addr: super::data_types::Word, data: super::data_types::Byte) {
        let lower_bytes = addr & 0x00FF;
        match lower_bytes {
            0x00 => {
                self.current_state.vsync = data > 0;
                println!(
                    "VSYNC requested: {}, {}",
                    self.current_state.vsync, self.current_clock_count
                );
                if !self.current_state.vsync {
                    self.render_next_tick = true
                }
            }
            0x01 => {
                self.current_state.vblank = data & 0x2 > 0;
                println!(
                    "VBLANK requested: {}, {}",
                    self.current_state.vblank, self.current_clock_count
                );
            }
            0x02 => {
                //println!("WSYNC requested");
            }
            0x09 => {
                self.current_state.bk_color = data.into();
                println!(
                    "Setting background color. Color set to {:x} {}, {}",
                    data, self.current_state.bk_color, self.current_clock_count
                )
            }
            0x1d => {
                //println!("Toggle missle 0");
                self.current_state.missile_0 = data & 0x2 > 0;
            }
            0x1e => {
                //println!("Toggle missle 1");
                self.current_state.missile_1 = data & 0x2 > 0;
            }
            0x1f => {
                //println!("Toggle ball");
                self.current_state.ball = data & 0x2 > 0;
            }
            _ => (), //println!("Not implemented"),
        };
    }
}
