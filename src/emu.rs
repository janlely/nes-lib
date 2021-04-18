#[wasm_bindgen]
struct Emu {

}

impl Emu {

    pub fn load_rom() {

    }

    pub fn frame() -> Vec<Color> {

    }
}

#[wasm_bindgen]
struct Color {
    red: u8,
    greed: u8,
    blue: u8,
}