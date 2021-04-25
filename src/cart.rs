use std::fs::File;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Seek;
use std::vec::Vec;
use crate::utils;
use anyhow::{anyhow, Result};


static SRAM_FLAG: u8 = 0x02;
struct RomHeader {
    prg: u8,
    chr: u8,
    flag6: u8,
    flag7: u8,
    flag8: u8,
    flag9: u8,
    flag10: u8
}

#[derive(Clone, Copy, Debug)]
pub enum MirrorType {
    HORIZONTAL,
    VERTICAL,
    NONE
}

pub struct Cartridge {
    mapper_code: u8,
    prg: Vec<u8>,
    chr: Vec<u8>,
    sram: Option<Vec<u8>>,
    prg_mirror: bool,
    has_sram: bool,
    mirror_type: MirrorType
}

impl Cartridge {

    pub fn new() -> Self {
        Cartridge {
            mapper_code: 0,
            prg: Vec::new(),
            chr: Vec::new(),
            sram: None,
            prg_mirror: false,
            has_sram: false,
            mirror_type: MirrorType::NONE
        }
    }
    pub fn load_from_file(&mut self, path: &str) ->Result<()> {
        let mut rom_file = File::open(path)?;
        //parse header
        let mut buffer = [0; 16];
        let n = rom_file.read(&mut buffer)?;
        if n != 16 {
            return Err(anyhow!("error reading rom header"));
        }
        let rom_header = RomHeader {
            prg: buffer[4],
            chr: buffer[5],
            flag6: buffer[6],
            flag7: buffer[7],
            flag8: buffer[8],
            flag9: buffer[9],
            flag10: buffer[10],
        };
        //mapper number
        let mapper_code = (rom_header.flag7 & 0xF0) | ((rom_header.flag6 & 0xF0) >> 4);
        //if trainer present
        if utils::binaryBoolAnd(rom_header.flag6, 0x04) {
            rom_file.seek(SeekFrom::Current(512))?;
        }

        //read prg data
        if rom_header.prg == 0 || rom_header.prg > 2 {
            return Err(anyhow!("invalied prg unit count: {}", rom_header.prg));
        }
        debug!("PRG unit size: {}", rom_header.prg);
        let prg_mirror = if rom_header.prg == 1 {true} else {false};
        let mut prg_buffer = vec![0u8; 16 * 1024 * rom_header.prg as usize];
        rom_file.read_exact(&mut prg_buffer)?;
        //read chr data
        if rom_header.chr == 0 {
            return Err(anyhow!("no CHR data in rom file"));
        }
        debug!("CHR unit size: {}", rom_header.chr);
        let mut chr_buffer = vec![0u8; 8 * 1024 * rom_header.chr as usize];
        rom_file.read_exact(&mut chr_buffer)?;
        //nametable mirror type
        let mirror_type = match rom_header.flag6 & 0x09 {
            8 =>  Ok(MirrorType::NONE),
            0 =>  Ok(MirrorType::HORIZONTAL),
            1 =>  Ok(MirrorType::VERTICAL),
            _ => Err(anyhow!("unknown mirror type, flag6: {:#06x}", rom_header.flag6))
        }?;
        self.mapper_code = mapper_code;
        self.prg = prg_buffer;
        self.chr = chr_buffer;
        self.prg_mirror = prg_mirror;
        self.has_sram = utils::binaryBoolAnd(rom_header.flag6, SRAM_FLAG);
        self.mirror_type = mirror_type;
        if self.has_sram {
            self.sram = Some(Vec::with_capacity(2*1024));
        }
        Ok(())
    }

    fn nrom_mapper_read_prg(&self, addr: u16) -> Result<u8> {
        if self.prg_mirror {
            let idx = (addr - 0x8000) & 0x3fff;
            Ok(self.prg[idx as usize])
        }else {
            let idx = addr - 0x8000;
            Ok(self.prg[idx as usize])
        }
    }
    fn nrom_mapper_read_chr(&self, addr: u16) -> Result<u8> {
        Ok(self.chr[addr as usize])
    }
    pub fn read_prg(&self, addr: u16) -> Result<u8> {
        match self.mapper_code {
            0 => self.nrom_mapper_read_prg(addr),
            _ => Err(anyhow!("unknown mapper type: {:#04x}", self.mapper_code))
        }
    }

    pub fn read_sram(&self, addr: u16) -> Result<u8> {
        if !self.has_sram {
            return Err(anyhow!("address not readable, {:#06x}", addr));
        }
        Ok(self.sram.unwrap()[addr as usize - 0x6000])
    }

    pub fn write_sram(&self, addr: u16, data: u8) -> Result<()>{
        if !self.has_sram {
            return Err(anyhow!("address not writeable, {:#06x}", addr));
        }
        self.sram.unwrap()[addr as usize - 0x6000] = data;
        Ok(())
    }

    pub fn read_chr(&self, addr: u16) -> Result<u8>{
        match self.mapper_code {
            0 => self.nrom_mapper_read_chr(addr),
            _ => Err(anyhow!("unknown mapper type: {:#04x}", self.mapper_code))
        }
    }

    pub fn get_mirror_type(&self) -> MirrorType {
        self.mirror_type
    }
}

