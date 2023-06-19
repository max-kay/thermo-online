use phases::{Array2d, BinAtom as Atom, BinConcentration as Concentration, System};
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
            entropy: Vec<f32>,
            free_energy: Vec<f32>,
            zip_data: Option<Vec<u8>>,
            c_0: f32,
            c_1: f32,
        }

        #[wasm_bindgen]
        impl $name {
            pub fn new(
                energies: Vec<f32>,
                c_0: f64,
                c_1: f64,
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
                    c_0: (c_0 / (c_0 + c_1)) as f32,
                    c_1: (c_1 / (c_0 + c_1)) as f32,
                    system: System::new(energies, None, Concentration::new(c_0, c_1)),
                    method,
                    encoder,
                    temp: Vec::with_capacity(log_capacity as usize),
                    int_energy: Vec::with_capacity(log_capacity as usize),
                    heat_capacity: Vec::with_capacity(log_capacity as usize),
                    acceptance_rate: Vec::with_capacity(log_capacity as usize),
                    entropy: Vec::with_capacity(log_capacity as usize),
                    free_energy: Vec::with_capacity(log_capacity as usize),
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
                        self.encoder.write_frame(&frame).expect("gif should be writable");
                    }
                }
                self.int_energy.push(stats.avg() / ($size * $size) as f32);
                self.heat_capacity
                    .push(stats.variance() / (temp * temp) / ($size * $size) as f32);
                self.acceptance_rate
                    .push(accepted as f32 / tot_steps as f32);
            }

            pub fn do_data_analysis(&mut self) {
                self.calc_entropy();
                self.calc_free_energy();
            }

            fn calc_entropy(&mut self) {
                if !self.entropy.is_empty() {
                    self.entropy = Vec::with_capacity(self.heat_capacity.len());
                }
                for i in 1..self.heat_capacity.len() {
                    let avg = (self.heat_capacity[i-1] / self.temp[i-1]
                         + self.heat_capacity[i] / self.temp[i])
                         / 2.0;
                    let last = self.entropy.last().copied().unwrap_or(-(self.c_0.ln() * self.c_0 + self.c_1.ln() * self.c_1));
                    if avg.is_finite() {
                        self.entropy.push(last + (self.temp[i] - self.temp[i-1]) * avg);
                    } else {
                        self.entropy.push(last)
                    }
                }
                let last = self.entropy.last().expect("should have at least one element.");
                self.entropy.push(*last)
            }

            fn calc_free_energy(&mut self) {
                for i in 0..self.int_energy.len() {
                    self.free_energy.push(self.int_energy[i]-self.temp[i]*self.entropy[i])
                }
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
            /// this function is not safe before calling `do_data_analysis`
            pub fn entropy_ptr(&self) -> *const f32 {
                self.entropy.as_ptr()
            }
            /// this function is not safe before calling `do_data_analysis`
            pub fn free_energy_ptr(&self) -> *const f32 {
                self.free_energy.as_ptr()
            }
        }

        #[wasm_bindgen]
        impl $name {
            pub fn make_zip(&mut self, method: &str, temp_steps: u32, start_temp: f32, e_steps: u32, m_steps: u32) {
                let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
                let options = FileOptions::default()
                                    .compression_method(zip::CompressionMethod::Deflated)
                                    .unix_permissions(0o755);
                zip.start_file("animation.gif", options).expect("error creating zip");
                zip.write_all(&self.encoder.get_ref()).expect("error creating zip");
                zip.start_file("model_output.csv", options).expect("error creating zip");
                zip.write_all(&self.make_csv_buffer()).expect("error creating zip");
                zip.start_file("model_params.txt", options).expect("error creating zip");
                zip.write_all(&self.make_model_params(method, temp_steps, start_temp, e_steps, m_steps)).expect("error creating zip");
                self.zip_data = Some(zip.finish().expect("error creating zip").into_inner());
            }

            fn make_csv_buffer(&self) -> Vec<u8> {
                let mut csv = Vec::new();
                writeln!(
                    csv,
                    "This file was generated as output to Thermodynamic Models https://max-kay.github.io/thermo-online/"
                )
                .expect("error writing csv header");
                writeln!(
                    csv,
                    "All extensive variable are divided by the number of lattice sites."
                )
                .expect("error writing csv header");
                writeln!(
                    csv,
                    "Code for website: https://github.com/max-kay/thermo-online"
                )
                .expect("error writing csv header");
                writeln!(
                    csv,
                    "Rust library used for the simulation: https://github.com/max-kay/phases"
                )
                .expect("error writing csv header");
                writeln!(
                    csv,
                    "temperature,internal_energy,heat_capacity,acceptance_rate,entropy,free_energy"
                )
                .expect("error writing csv header");
                for i in 0..self.int_energy.len(){
                    writeln!(
                        csv,
                        "{},{},{},{},{},{},",
                        self.temp[i],
                        self.int_energy[i],
                        self.heat_capacity[i],
                        self.acceptance_rate[i],
                        self.entropy[i],
                        self.free_energy[i],
                    ).expect("error writing cvs")
                }
                csv
            }

            fn make_model_params(&self, method: &str, temp_steps: u32, start_temp: f32, e_steps: u32, m_steps: u32) -> Vec<u8> {
                let mut buf = Vec::new();
                writeln!(buf, "Model Size: {}", $size).expect("error creating model parms file");
                writeln!(buf, "Method: {}", method).expect("error creating model parms file");
                writeln!(buf, "Energies: {}", self.system.get_energies_dict()).expect("error creating model parms file");
                writeln!(buf, "Concentration A: {}, Concentration B: {}", self.c_0, self.c_1).expect("error creating model parms file");
                writeln!(buf, "Temperature Steps: {}", temp_steps).expect("error creating model parms file");
                writeln!(buf, "Start Temperature: {}", start_temp).expect("error creating model parms file");
                writeln!(buf, "Steps for Equilibrium: {}", e_steps).expect("error creating model parms file");
                writeln!(buf, "Steps for Measurement: {}", m_steps).expect("error creating model parms file");
                buf
            }

            pub fn get_zip_ptr(&self) -> *const u8 {
                self.zip_data.as_ref().expect("zip data doesnt exist").as_ptr()
            }

            pub fn get_zip_len(&self) -> usize {
                self.zip_data.as_ref().expect("zip data doesnt exist").len()
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
makemodel!(XSmallModel, 16);
