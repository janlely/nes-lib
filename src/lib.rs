extern crate wasm_bindgen;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
use wasm_bindgen::prelude::*;
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::rc::Rc;
#[macro_use]
extern crate lazy_static;

mod cpu;
mod ppu;
mod cart;
mod utils;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    // log(format!("hello {}", name).as_str());
    println!("hello {}!", name);
}

#[wasm_bindgen]
struct Emu {
    cart: Rc<RefCell<cart::Cartridge>>,
    cpu: Rc<RefCell<cpu::CPU>>,
    ppu: Rc<RefCell<ppu::PPU>>,
    frame: Rc<RefCell<Vec<u8>>>,
    cycles: usize
}

impl Emu {
    pub fn new() -> Self {
        let frame = Rc::new(RefCell::new(Vec::with_capacity(240*256)));
        let cart_rc = Rc::new(RefCell::new(cart::Cartridge::new()));
        let ppu_rc = Rc::new(RefCell::new(ppu::PPU::new(frame.clone()))) ;
        let cpu_rc = Rc::new(RefCell::new(cpu::CPU::new(cart_rc.clone(), ppu_rc.clone())));
        ppu_rc.borrow_mut().set_vblank_cb(|| cpu_rc.borrow_mut().interrupt(cpu::InteruptType::NMI));
        Emu {
            cart: cart_rc.clone(),
            cpu: cpu_rc.clone(),
            ppu: ppu_rc.clone(),
            frame: frame.clone(),
            cycles: 0
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<()>{
        self.cart.borrow_mut().load_from_file(path)
    }

    pub fn init(&mut self) -> Result<()> {
        self.cpu.borrow_mut().reset()?;
        self.ppu.borrow_mut().reset()?;
        Ok(())
    }

    pub fn tick(&mut self) {
        if self.cycles % 4 == 0 {
            self.cpu.borrow_mut().step();
        }else{
            self.ppu.borrow_mut().step();
        }
        self.cycles += 1;
    }

    pub fn frame(&mut self) -> Result<Rc<RefCell<Vec<u8>>>> {
        while !self.cpu.borrow().is_frame_ready() {
            self.tick();
        }
        return Ok(self.frame.clone())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
