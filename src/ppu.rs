use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::rc::Rc;
use crate::utils;
use crate::cart;

pub struct PPU {
    vram: Vec<u8>,
    oam: Vec<u8>,
    palette: Vec<u8>,
    frame: Rc<RefCell<Vec<u8>>>,
    cycles: u16,
    scanline: u16,
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
            vram: Vec::with_capacity(0xFFF as usize),
            oam: Vec::with_capacity(512),
            palette: Vec::with_capacity(0x20 as usize),
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
    fn set_scroll(&mut self, data: u8) {
        if self.scroll_first_write {
            self.x_scroll = if self.r_ppuctrl & 0x1 == 1 {256 + data as u16} else {data as u16};
        }else {
            self.y_scroll = if self.r_ppuctrl & 0x2 == 1 {240+ data as u16} else {data as u16};
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

    pub fn pre_rendering(&mut self) -> Result<()> {
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
    pub fn rendering(&mut self) -> Result<()> {
        if self.cycles == 0 {
            // idle cycle
            return Ok(())
        }
        if self.cycles > 0 && self.cycles <= 256 {
            // background pixel
            let x = self.cycles - 1 + self.x_scroll;
            let y = self.scanline + self.y_scroll;
            let tile_idx: u16 = ((y >> 3) << 5) + (x >> 3);
            let tile_row_addr_low =
                (((self.r_ppuctrl & 0x10) as u16) << 4) |
                ((y >> 3) << 8) |
                ((x >> 3) << 4)  |
                (y & 0x0007);
            let tile_row_addr_upper =
                (((self.r_ppuctrl & 0x10) as u16) << 4) |
                ((y >> 3) << 8) |
                ((x >> 3) << 4)  |
                0x0008 |
                (y & 0x0007);
            let addribute_table_addr =
                
                (((self.y_scroll >> 5) << 3) as u16) |
                (((self.x_scroll >> 5) << 3) as u16)
            let tile_row_low_byte = self.read(tile_row_addr_low)?;
            let tile_row_upper_byte = self.read(tile_row_addr_upper)?;
            let palette_addr = 0x3F00 | 0x0010 |  
            
        }
        self.cycles += 1;
        Ok(())
    }
    pub fn post_rendering(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn show_background(&self) -> bool {
        return utils::binaryBoolAnd(self.r_ppumask, 0x10);
    }
    fn show_sprites(&self) -> bool {
        return utils::binaryBoolAnd(self.r_ppumask, 0x08);
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
        self.cart.borrow().read_chr(addr)
    }
    
    fn read_nametable(&self, addr: u16) -> Result<u8> {
        Ok(self.vram[(addr & 0x7FF) as usize])
    }
    
    fn read_palette(&self, addr: u16) -> Result<u8> {
        Ok(self.palette[(addr & 0x1F) as usize])
    }
}

enum Stage {
    PreRendering,
    Rendering,
    PostRendering
}