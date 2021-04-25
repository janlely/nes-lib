use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::rc::Rc;
use crate::cpu;
use crate::utils;
use crate::cart;

pub struct PPU {
    vram: Vec<u8>,
    oam: Vec<u8>,
    frame: Rc<RefCell<Vec<u8>>>,
    cycles: usize,
    scanline: u32,
    // registers
    r_ppuctrl: u8,
    r_ppumask: u8,
    r_ppustatus: u8,
    r_oamaddr: u8,
    r_oamdata: u8,
    r_ppuaddr: u8,
    r_ppudata: u8,
    r_oamdma: u8,
    x_scroll: u16,
    y_scroll: u16,
    scroll_first_write: bool,

    even_frame: bool,
    stage: Stage,
    
    cart: Rc<RefCell<cart::Cartridge>>,
    vblank_cb: Option<Box<dyn Fn()>>
}

impl PPU {

    pub fn new(frame: Rc<RefCell<Vec<u8>>>, cart: Rc<RefCell<cart::Cartridge>>) -> Self {
        PPU {
            vram: Vec::new(),
            oam: Vec::new(),
            frame: frame,
            cycles: 0,
            scanline: 0,
            // registers
            r_ppuctrl: 0,
            r_ppumask: 0,
            r_ppustatus: 0,
            r_oamaddr: 0,
            r_oamdata: 0,
            r_ppuaddr: 0,
            r_ppudata: 0,
            r_oamdma: 0,
            x_scroll: 0,
            y_scroll: 0,
            scroll_first_write: true,
            even_frame: true,
            stage: Stage::PreRendering,
            cart: cart,
            vblank_cb: None
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        debug!("initializing ppu");
        Ok(())
    }

    pub fn set_vblank_cb(&mut self, cb: Box<dyn Fn()>) {
        self.vblank_cb = Some(cb);
    }

    pub fn step(&mut self) -> Result<()> {
        match self.stage {
            Stage::PreRendering =>  self.pre_rendering(),
            Stage::Rendering => self.rendering(),
            Stage::PostRendering => self.post_rendering()
        }
    }

    pub fn pre_rendering(&mut self) -> result<()> {
        if self.end_prerendering() {
            self.cycles = 0;
            self.scanline = 0;
            self.stage = Stage::Rendering;
        }
        self.cycles += 1;
        Ok(())
    }
    fn end_prerendering(&self) -> bool {
        if self.cycles == 340 {
            return true;
        }
        if self.cycles == 339 && !self.even_frame && self.show_background() && self.show_sprites() {
            return true;
        } 
        false
    }
    pub fn rendering(&mut self) -> result<()> {
        if self.cycles == 0 {
            // idle cycle
            return Ok(())
        }
        if self.cycles > 0 && self.cycles <= 256 {
            // tile index
            let tile: u16 = ((self.y_scroll >> 3) << 5) + (self.x_scroll >> 8);
        }
        self.cycles += 1;
        Ok(())
    }
    pub fn post_rendering(&mut self) -> result<()> {
        Ok(())
    }
    
    fn show_background(&self) -> bool {
        utils::binaryBoolAnd(self.r_ppumask, 0x10);
    }
    fn show_sprites(&self) -> bool {
        utils::binaryBoolAnd(self.r_ppumask, 0x08);
    }

    fn read(&self, addr: u16) -> Result<u8> {
        match addr {
            addr if addr < 0x2000 => self.read_chr(addr),
            addr if addr < 0x3f00 => self.read_nametable(addr),
            addr if addr < 0x4000 => self.read_palette(addr),
            addr => anyhow!("unknown ppu address: {}", addr)
        }
    }
    
    fn read_chr(&self, addr: u16) -> Result<u8> {

    }
}

enum Stage {
    PreRendering,
    Rendering,
    PostRendering
}