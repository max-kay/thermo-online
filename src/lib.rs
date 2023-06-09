use phases::{Array2d, NumAtom, NumC, System};
use wasm_bindgen::prelude::*;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {name}!"));
}

type Concentration = NumC<2>;
type Atom = NumAtom<2>;

type MediumSystem = System<Array2d<Atom, 128, 128>, [f32; 4]>;

#[wasm_bindgen]
pub struct MediumModel {
    system: MediumSystem,
    method: fn(&mut MediumSystem, f32) -> bool,
    animation: Vec<[[u8; 128]; 128]>,
}

#[wasm_bindgen]
impl MediumModel {
    pub fn new(energies: Vec<f32>, c_1: f64, c_2: f64, method: &str) -> Self {
        let method = match method {
            "move_vacancy" => MediumSystem::move_vacancy,
            _ => {
                alert(&format!(
                    "method '{}', not found using 'move_vacancy'",
                    method
                ));
                MediumSystem::move_vacancy
            }
        };

        let energies = [energies[0], energies[1], energies[2], energies[3]];
        Self {
            system: MediumSystem::new(energies, None, Concentration::new([c_1, c_2])),
            method,
            animation: Vec::new(),
        }
    }

    pub fn take_steps(&mut self, steps: u32, temp: f32) {
        for _ in 0..steps {
            (self.method)(&mut self.system, temp);
            // SAFETY the implentation of NumAtom guarantees this is safe
            unsafe {
                self.animation
                    .push(std::mem::transmute(*self.system.return_state().grid))
            }
        }
    }
}

/// Animation stuff
#[wasm_bindgen]
impl MediumModel {
    pub fn width(&self) -> u32 {
        128
    }

    pub fn height(&self) -> u32 {
        128
    }

    pub fn anim_len(&self) -> u32 {
        self.animation.len() as u32
    }

    pub fn anim_ptr(&self) -> *const u8 {
        self.animation[0].as_ptr().cast()
    }
}
