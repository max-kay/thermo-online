use phases::{Array2d, NumAtom, NumC, System};
use std::io::{Cursor, Write};
use wasm_bindgen::prelude::*;
use zip::write::{FileOptions, ZipWriter};
mod utils;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

const PALETTE: &[u8] = phases::anim::PALETTE;

#[wasm_bindgen]
pub fn get_color(num: u8) -> String {
    format!(
        "rgb({}, {}, {})",
        PALETTE[3 * num as usize],
        PALETTE[(3 * num + 1) as usize],
        PALETTE[(3 * num + 2) as usize],
    )
}

#[allow(unused)]
macro_rules! console {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

#[wasm_bindgen]
pub fn start_logs() {
    utils::set_panic_hook()
}

#[wasm_bindgen]
pub fn make_energies(j00: f32, j01: f32, j11: f32) -> Vec<f32> {
    vec![j00, j01, j01, j11]
}

type Concentration = NumC<2>;
type Atom = NumAtom<2>;

macro_rules! makemodel {
    ($name:ident, $size:literal) => {
        #[wasm_bindgen]
        pub struct $name {
            system: System<Array2d<Atom, $size, $size>, [f32; 4]>,
            method: fn(&mut System<Array2d<Atom, $size, $size>, [f32; 4]>, f32) -> bool,
            encoder: gif::Encoder<Vec<u8>>,
            temp: Vec<f32>,
            int_energy: Vec<f32>,
            heat_capacity: Vec<f32>,
            acceptance_rate: Vec<f32>,
            zip_data: Option<Vec<u8>>,
        }

        #[wasm_bindgen]
        impl $name {
            pub fn new(
                energies: Vec<f32>,
                c_1: f64,
                c_2: f64,
                method: &str,
                log_capacity: u32,
                gif_delay: u16,
            ) -> Self {
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
                let mut palette = PALETTE.to_vec();
                palette[4 * 3] = 0;
                palette[4 * 3 + 1] = 0;
                palette[4 * 3 + 2] = 0;
                let encoder =
                    phases::anim::prepare_vec_encoder($size, $size, Some(gif_delay), &palette);
                let energies = [energies[0], energies[1], energies[2], energies[3]];
                Self {
                    system: System::new(energies, None, Concentration::new([c_1, c_2])),
                    method,
                    encoder,
                    temp: Vec::with_capacity(log_capacity as usize),
                    int_energy: Vec::with_capacity(log_capacity as usize),
                    heat_capacity: Vec::with_capacity(log_capacity as usize),
                    acceptance_rate: Vec::with_capacity(log_capacity as usize),
                    zip_data: None,
                }
            }

            pub fn run_at_temp(
                &mut self,
                equilibrium_steps: u32,
                measurement_steps: u32,
                temp: f32,
                frames: u32,
            ) {
                let tot_steps = measurement_steps * $size * $size;
                self.temp.push(temp);
                for _ in 0..equilibrium_steps * $size * $size {
                    (self.method)(&mut self.system, 1.0 / temp);
                }

                let mut stats = phases::StreamingVariance::new();
                let mut accepted = 0;
                for k in 0..tot_steps {
                    accepted += (self.method)(&mut self.system, 1.0 / temp) as u32;
                    stats.add_value(self.system.internal_energy());
                    if k % (tot_steps / frames) == 0 {
                        let frame = self.system.get_frame();
                        self.encoder.write_frame(&frame).unwrap();
                    }
                }
                self.int_energy.push(stats.avg() / ($size * $size) as f32);
                self.heat_capacity
                    .push(stats.variance() / (temp * temp) / ($size * $size) as f32);
                self.acceptance_rate
                    .push(accepted as f32 / tot_steps as f32);
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
            pub fn acceptance_rate_ptr(&self) -> *const f32 {
                self.acceptance_rate.as_ptr()
            }
        }

        #[wasm_bindgen]
        impl $name {
            pub fn get_zip_ptr(&mut self) -> *const u8 {
                let mut csv = Vec::new();
                writeln!(csv, "This file was generated as output to Thermodynamic Models by Max Krummenacher (mkrummenache@student.ethz.ch)").unwrap();
                writeln!(csv, "Temperature,Internal Energy,Heat Capacity,Acceptance Rate").unwrap();
                for i in 0..self.int_energy.len(){
                    writeln!(csv, "{},{},{},{}", self.temp[i], self.int_energy[i], self.heat_capacity[i], self.acceptance_rate[i]).unwrap()
                }
                let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
                let options = FileOptions::default()
                                    .compression_method(zip::CompressionMethod::Deflated)
                                    .unix_permissions(0o755);
                zip.start_file("animation.gif", options).unwrap();
                zip.write_all(&self.encoder.get_ref()).unwrap();
                zip.start_file("model_output.csv", options).unwrap();
                zip.write_all(&csv).unwrap();
                zip.finish().unwrap();
                self.zip_data = Some(zip.finish().unwrap().into_inner());
                self.zip_data.as_ref().unwrap().as_ptr()
            }

            pub fn destory_zip_data(&mut self) {
                self.zip_data = None;
            }
        }

        #[wasm_bindgen]
        impl $name {
            pub fn width(&self) -> u32 {
                $size
            }

            pub fn height(&self) -> u32 {
                $size
            }

            pub fn gif_len(&self) -> u32 {
                self.encoder.get_ref().len() as u32
            }

            pub fn gif_ptr(&self) -> *const u8 {
                self.encoder.get_ref().as_ptr()
            }
        }
    };
}

makemodel!(XBigModel, 256);
makemodel!(BigModel, 128);
makemodel!(MediumModel, 64);
makemodel!(SmallModel, 32);
makemodel!(XSmallModel, 32);
