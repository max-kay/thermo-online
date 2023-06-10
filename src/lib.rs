use phases::{Array2d, NumAtom, NumC, System};
use wasm_bindgen::prelude::*;
mod utils;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

type Concentration = NumC<2>;
type Atom = NumAtom<2>;

macro_rules! makemodel {
    ($name:ident, $size:literal) => {
        #[wasm_bindgen]
        pub struct $name {
            system: System<Array2d<Atom, $size, $size>, [f32; 4]>,
            method: fn(&mut System<Array2d<Atom, $size, $size>, [f32; 4]>, f32) -> bool,
            animation: Vec<[[u8; $size]; $size]>,
            temp: Vec<f32>,
            int_energy: Vec<f32>,
            heat_capacity: Vec<f32>,
        }

        #[wasm_bindgen]
        impl $name {
            pub fn new(
                energies: Vec<f32>,
                c_1: f64,
                c_2: f64,
                method: &str,
                animation_capcity: u32,
                log_capacity: u32,
            ) -> Self {
                utils::set_panic_hook();
                let method = match method {
                    "monte_carlo_swap" => System::monte_carlo_swap,
                    "move_vacancy" => System::move_vacancy,
                    _ => {
                        alert(&format!(
                            "method '{}' not found, using 'move_vacancy'",
                            method
                        ));
                        System::move_vacancy
                    }
                };

                let energies = [energies[0], energies[1], energies[2], energies[3]];
                Self {
                    system: System::new(energies, None, Concentration::new([c_1, c_2])),
                    method,
                    animation: Vec::with_capacity(animation_capcity as usize),
                    temp: Vec::with_capacity(log_capacity as usize),
                    int_energy: Vec::with_capacity(log_capacity as usize),
                    heat_capacity: Vec::with_capacity(log_capacity as usize),
                }
            }

            pub fn run_at_temp(
                &mut self,
                equilibrium_steps: u32,
                measurement_steps: u32,
                temp: f32,
                frames: u32,
            ) {
                self.temp.push(temp);

                for _ in 0..equilibrium_steps * $size * $size {
                    (self.method)(&mut self.system, temp);
                }

                // variance after https://math.stackexchange.com/questions/20593/calculate-variance-from-a-stream-of-sample-values
                let mut m_k = 0.0;
                let mut m_k_1;
                let mut v_k = 0.0;
                let mut v_k_1;

                for k in 1..=measurement_steps * $size * $size {
                    (self.method)(&mut self.system, temp);
                    let x_k = self.system.internal_energy();
                    m_k_1 = m_k;
                    v_k_1 = v_k;
                    m_k = m_k_1 + (x_k - m_k_1) / k as f32;
                    v_k = v_k_1 - (x_k - m_k_1) * (x_k - m_k);
                    if k % measurement_steps / frames == 0 {
                        self.animation.push(
                            // SAFETY the implentation of NumAtom guarantees this is safe
                            unsafe { std::mem::transmute(*self.system.return_state().grid) },
                        )
                    }
                }
                self.int_energy.push(m_k);
                let variance = v_k / (measurement_steps * $size * $size) as f32;
                self.heat_capacity.push(variance / temp / temp)
            }
        }

        #[wasm_bindgen]
        impl $name {
            pub fn log_len(&self) -> u32 {
                self.int_energy.len() as u32
            }

            pub fn int_energy_ptr(&self) -> *const f32 {
                self.int_energy.as_ptr()
            }
            pub fn temp_ptr(&self) -> *const f32 {
                self.temp.as_ptr()
            }
            pub fn heat_capacity_ptr(&self) -> *const f32 {
                self.heat_capacity.as_ptr()
            }
        }

        /// Animation stuff
        #[wasm_bindgen]
        impl $name {
            pub fn width(&self) -> u32 {
                $size
            }

            pub fn height(&self) -> u32 {
                $size
            }

            pub fn anim_len(&self) -> u32 {
                self.animation.len() as u32
            }

            pub fn anim_start_ptr(&self) -> *const u8 {
                self.animation[0].as_ptr().cast()
            }

            pub fn anim_frame_ptr(&self, idx: usize) -> *const u8 {
                self.animation[idx].as_ptr().cast()
            }

            pub fn anim_last_frame_ptr(&self) -> *const u8 {
                self.animation[self.animation.len() - 1].as_ptr().cast()
            }
        }
    };
}

makemodel!(BigModel, 256);
makemodel!(MediumModel, 128);
makemodel!(SmallModel, 64);
#[wasm_bindgen]
pub struct TinyModel {
    system: System<Array2d<Atom, 32, 32>, [f32; 4]>,
    method: fn(&mut System<Array2d<Atom, 32, 32>, [f32; 4]>, f32) -> bool,
    animation: Vec<[[u8; 32]; 32]>,
    temp: Vec<f32>,
    int_energy: Vec<f32>,
    heat_capacity: Vec<f32>,
}
#[wasm_bindgen]
impl TinyModel {
    pub fn new(
        energies: Vec<f32>,
        c_1: f64,
        c_2: f64,
        method: &str,
        animation_capcity: u32,
        log_capacity: u32,
    ) -> Self {
        utils::set_panic_hook();
        let method = match method {
            "monte_carlo_swap" => System::monte_carlo_swap,
            "move_vacancy" => System::move_vacancy,
            _ => {
                alert(&format!(
                    "method '{}' not found, using 'move_vacancy'",
                    method
                ));
                System::move_vacancy
            }
        };
        let energies = [energies[0], energies[1], energies[2], energies[3]];
        Self {
            system: System::new(energies, None, Concentration::new([c_1, c_2])),
            method,
            animation: Vec::with_capacity(animation_capcity as usize),
            temp: Vec::with_capacity(log_capacity as usize),
            int_energy: Vec::with_capacity(log_capacity as usize),
            heat_capacity: Vec::with_capacity(log_capacity as usize),
        }
    }
    pub fn run_at_temp(
        &mut self,
        equilibrium_steps: u32,
        measurement_steps: u32,
        temp: f32,
        frames: u32,
    ) {
        self.temp.push(temp);
        for _ in 0..equilibrium_steps * 32 * 32 {
            (self.method)(&mut self.system, temp);
        }
        let mut m_k = 0.0;
        let mut m_k_1;
        let mut v_k = 0.0;
        let mut v_k_1;
        for k in 1..=measurement_steps * 32 * 32 {
            (self.method)(&mut self.system, temp);
            let x_k = self.system.internal_energy();
            m_k_1 = m_k;
            v_k_1 = v_k;
            m_k = m_k_1 + (x_k - m_k_1) / k as f32;
            v_k = v_k_1 - (x_k - m_k_1) * (x_k - m_k);
            if k % measurement_steps / frames == 0 {
                self.animation
                    .push(unsafe { std::mem::transmute(*self.system.return_state().grid) })
            }
        }
        self.int_energy.push(m_k);
        let variance = v_k / (measurement_steps * 32 * 32) as f32;
        self.heat_capacity.push(variance / temp / temp)
    }
}
#[wasm_bindgen]
impl TinyModel {
    pub fn log_len(&self) -> u32 {
        self.int_energy.len() as u32
    }
    pub fn int_energy_ptr(&self) -> *const f32 {
        self.int_energy.as_ptr()
    }
    pub fn temp_ptr(&self) -> *const f32 {
        self.temp.as_ptr()
    }
    pub fn heat_capacity_ptr(&self) -> *const f32 {
        self.heat_capacity.as_ptr()
    }
}
#[doc = " Animation stuff"]
#[wasm_bindgen]
impl TinyModel {
    pub fn width(&self) -> u32 {
        32
    }
    pub fn height(&self) -> u32 {
        32
    }
    pub fn anim_len(&self) -> u32 {
        self.animation.len() as u32
    }
    pub fn anim_start_ptr(&self) -> *const u8 {
        self.animation[0].as_ptr().cast()
    }
    pub fn anim_frame_ptr(&self, idx: usize) -> *const u8 {
        self.animation[idx].as_ptr().cast()
    }
    pub fn anim_last_frame_ptr(&self) -> *const u8 {
        self
        .animation[self.animation.len() - 1]
        .as_ptr()
        .cast()
    }
}
