use phases::{
    disentangle, BinAtom as Atom, BinConcentration as Concentration, ClusterDistribution,
    ClusterStats, FastArray, System,
};
use std::io::{Cursor, Write};
use wasm_bindgen::prelude::*;
use zip::write::{FileOptions, ZipWriter};

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
pub fn make_energies(j00: f32, j01: f32, j11: f32) -> Vec<f32> {
    vec![j00, j01, j01, j11]
}

struct ModelResults {
    pub temp: Vec<f32>,
    pub int_energy: Vec<f32>,
    pub heat_capacity: Vec<f32>,
    pub acceptance_rate: Vec<f32>,
    pub entropy: Vec<f32>,
    pub free_energy: Vec<f32>,
    distr_0: Vec<ClusterDistribution>,
    distr_1: Vec<ClusterDistribution>,
    pub cluster_stats_0: [Vec<u32>; 5],
    pub cluster_stats_1: [Vec<u32>; 5],
    distrs_per_temp: u32,
}

impl ModelResults {
    pub fn new(capacity: usize, distrs_per_temp: u32) -> Self {
        Self {
            temp: Vec::with_capacity(capacity),
            int_energy: Vec::with_capacity(capacity),
            heat_capacity: Vec::with_capacity(capacity),
            acceptance_rate: Vec::with_capacity(capacity),
            entropy: Vec::with_capacity(capacity),
            free_energy: Vec::with_capacity(capacity),
            distr_0: Vec::with_capacity(capacity),
            distr_1: Vec::with_capacity(capacity),
            cluster_stats_0: Default::default(),
            cluster_stats_1: Default::default(),
            distrs_per_temp,
        }
    }

    pub fn push_temp(&mut self, value: f32) {
        self.temp.push(value)
    }
    pub fn push_int_energy(&mut self, value: f32) {
        self.int_energy.push(value)
    }
    pub fn push_heat_capacity(&mut self, value: f32) {
        self.heat_capacity.push(value)
    }
    pub fn push_acceptance_rate(&mut self, value: f32) {
        self.acceptance_rate.push(value)
    }
    pub fn push_distr_0(&mut self, value: ClusterDistribution) {
        self.distr_0.push(value)
    }
    pub fn push_distr_1(&mut self, value: ClusterDistribution) {
        self.distr_1.push(value)
    }
}
impl ModelResults {
    pub fn do_data_analysis(&mut self, ideal_entropy: f32) {
        self.calc_entropy(ideal_entropy);
        self.calc_free_energy();
        self.cluster_stats_0 = disentangle(
            self.distr_0
                .iter()
                .map(ClusterStats::from_map_atom)
                .collect(),
        );
        self.cluster_stats_1 = disentangle(
            self.distr_1
                .iter()
                .map(ClusterStats::from_map_atom)
                .collect(),
        );
    }

    fn calc_entropy(&mut self, ideal_entropy: f32) {
        if !self.entropy.is_empty() {
            self.entropy = Vec::with_capacity(self.heat_capacity.len());
        }
        self.entropy.push(ideal_entropy);
        for i in 1..self.heat_capacity.len() {
            let avg = (self.heat_capacity[i] / self.temp[i]
                + self.heat_capacity[i - 1] / self.temp[i - 1])
                / 2.0;
            let last = *self.entropy.last().unwrap();
            if avg.is_finite() {
                self.entropy
                    .push(last + (self.temp[i] - self.temp[i - 1]) * avg);
            } else {
                self.entropy.push(last)
            }
        }
    }

    fn calc_free_energy(&mut self) {
        for i in 0..self.int_energy.len() {
            self.free_energy
                .push(self.int_energy[i] - self.temp[i] * self.entropy[i])
        }
    }
}

impl ModelResults {
    pub fn len(&self) -> u32 {
        self.temp.len() as u32
    }
}

impl ModelResults {
    fn make_csv_buffer(&self) -> std::io::Result<Vec<u8>> {
        let mut csv = Vec::new();
        writeln!(
            csv,
            "This file was generated as output to Thermodynamic Models https://max-kay.github.io/thermo-online/"
        )?;
        writeln!(
            csv,
            "All extensive variable are divided by the number of lattice sites."
        )?;
        writeln!(
            csv,
            "Code for website: https://github.com/max-kay/thermo-online"
        )?;
        writeln!(
            csv,
            "Rust library used for the simulation: https://github.com/max-kay/phases"
        )?;
        writeln!(
            csv,
            "temperature,internal_energy,heat_capacity,acceptance_rate,entropy,free_energy"
        )?;
        for i in 0..self.int_energy.len() {
            writeln!(
                csv,
                "{},{},{},{},{},{},",
                self.temp[i],
                self.int_energy[i],
                self.heat_capacity[i],
                self.acceptance_rate[i],
                self.entropy[i],
                self.free_energy[i],
            )?
        }
        Ok(csv)
    }

    fn make_json_buffer(&self) -> std::io::Result<Vec<u8>> {
        fn remove_last_comma(buf: &mut Vec<u8>) {
            buf.pop();
            buf.pop();
            write!(buf, "\n").unwrap();
        }

        let mut buf = Vec::new();
        writeln!(buf, "[")?;

        writeln!(buf, "  {{")?;
        writeln!(buf, "    \"atom\": \"A\",")?;
        writeln!(buf, "    \"distributions\": [")?;
        for (temp, distr) in self.temp.iter().zip(self.distr_0.iter()) {
            writeln!(buf, "    {{")?;
            writeln!(buf, "        \"temperature\": {},", temp)?;

            writeln!(buf, "        \"distribution\": {{")?;
            for (size, count) in distr.ref_map().iter() {
                writeln!(
                    buf,
                    "          \"{}\": {},",
                    size,
                    *count as f32 / self.distrs_per_temp as f32
                )?;
            }
            remove_last_comma(&mut buf);
            writeln!(buf, "      }}")?;

            writeln!(buf, "    }},")?;
        }
        remove_last_comma(&mut buf);
        writeln!(buf, "    ]")?;
        writeln!(buf, "  }},")?;

        writeln!(buf, "  {{")?;
        writeln!(buf, "    \"atom\": \"B\",")?;
        writeln!(buf, "    \"distributions\": [")?;
        for (temp, distr) in self.temp.iter().zip(self.distr_1.iter()) {
            writeln!(buf, "    {{")?;
            writeln!(buf, "        \"temperature\": {},", temp)?;

            writeln!(buf, "        \"distribution\": {{")?;
            for (size, count) in distr.ref_map().iter() {
                writeln!(
                    buf,
                    "          \"{}\": {},",
                    size,
                    *count as f32 / self.distrs_per_temp as f32
                )?;
            }
            remove_last_comma(&mut buf);
            writeln!(buf, "      }}")?;

            writeln!(buf, "    }},")?;
        }
        remove_last_comma(&mut buf);
        writeln!(buf, "    ]")?;
        writeln!(buf, "  }}")?;

        writeln!(buf, "]")?;
        Ok(buf)
    }
}

macro_rules! makemodel {
    ($name:ident, $size:literal, $pow:literal) => {
        #[wasm_bindgen]
        pub struct $name {
            system: System<FastArray<Atom, $size, $pow>, [f32; 4]>,
            method: fn(&mut System<FastArray<Atom, $size, $pow>, [f32; 4]>, f32) -> bool,
            encoder: gif::Encoder<Vec<u8>>,
            results: ModelResults,
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
                distrs_per_temp: u32,
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
                    results: ModelResults::new(log_capacity as usize, distrs_per_temp),
                    zip_data: None,
                }
            }

            pub fn run_at_temp(
                &mut self,
                equilibrium_steps: u32,
                measurement_steps: u32,
                temp: f32,
                frames: u32,
                distrs_per_temp: u32,
            ) {
                self.results.push_temp(temp);
                for _ in 0..equilibrium_steps * $size * $size {
                    (self.method)(&mut self.system, 1.0 / temp);
                }

                let mut stats = phases::StreamingStats::new();
                let mut accepted = 0;
                let tot_steps = measurement_steps * $size * $size;
                let mut distr_0 = ClusterDistribution::new();
                let mut distr_1 = ClusterDistribution::new();
                for k in 0..tot_steps {
                    accepted += (self.method)(&mut self.system, 1.0 / temp) as u32;
                    stats.add_value(self.system.internal_energy());
                    if k % (tot_steps / frames) == 0 {
                        let frame = self.system.get_frame();
                        self.encoder
                            .write_frame(&frame)
                            .expect("gif should be writable");
                    }
                    if k % (tot_steps / distrs_per_temp) == 0 {
                        distr_0.combine(&self.system.count_clusters(Atom::new(0)));
                        distr_1.combine(&self.system.count_clusters(Atom::new(1)));
                    }
                }
                self.results
                    .push_int_energy(stats.avg() / ($size * $size) as f32);
                self.results
                    .push_heat_capacity(stats.variance() / (temp * temp) / ($size * $size) as f32);
                self.results
                    .push_acceptance_rate(accepted as f32 / tot_steps as f32);
                self.results.push_distr_0(distr_0);
                self.results.push_distr_1(distr_1);
            }

            pub fn do_data_analysis(&mut self) {
                self.results
                    .do_data_analysis(-(self.c_0.ln() * self.c_0 + self.c_1.ln() * self.c_1))
            }
        }

        #[wasm_bindgen]
        impl $name {
            pub fn log_len(&self) -> u32 {
                self.results.len() as u32
            }

            pub fn temp_ptr(&self) -> *const f32 {
                self.results.temp.as_ptr()
            }
            pub fn int_energy_ptr(&self) -> *const f32 {
                self.results.int_energy.as_ptr()
            }
            pub fn heat_capacity_ptr(&self) -> *const f32 {
                self.results.heat_capacity.as_ptr()
            }
            pub fn acceptance_rate_ptr(&self) -> *const f32 {
                self.results.acceptance_rate.as_ptr()
            }
            pub fn entropy_ptr(&self) -> *const f32 {
                self.results.entropy.as_ptr()
            }
            pub fn free_energy_ptr(&self) -> *const f32 {
                self.results.free_energy.as_ptr()
            }

            pub fn cs_0_min_ptr(&self) -> *const u32 {
                self.results.cluster_stats_0[0].as_ptr()
            }
            pub fn cs_0_q1_ptr(&self) -> *const u32 {
                self.results.cluster_stats_0[1].as_ptr()
            }
            pub fn cs_0_mean_ptr(&self) -> *const u32 {
                self.results.cluster_stats_0[2].as_ptr()
            }
            pub fn cs_0_q3_ptr(&self) -> *const u32 {
                self.results.cluster_stats_0[3].as_ptr()
            }
            pub fn cs_0_max_ptr(&self) -> *const u32 {
                self.results.cluster_stats_0[4].as_ptr()
            }

            pub fn cs_1_min_ptr(&self) -> *const u32 {
                self.results.cluster_stats_1[0].as_ptr()
            }
            pub fn cs_1_q1_ptr(&self) -> *const u32 {
                self.results.cluster_stats_1[1].as_ptr()
            }
            pub fn cs_1_mean_ptr(&self) -> *const u32 {
                self.results.cluster_stats_1[2].as_ptr()
            }
            pub fn cs_1_q3_ptr(&self) -> *const u32 {
                self.results.cluster_stats_1[3].as_ptr()
            }
            pub fn cs_1_max_ptr(&self) -> *const u32 {
                self.results.cluster_stats_1[4].as_ptr()
            }
        }

        #[wasm_bindgen]
        impl $name {
            pub fn make_zip(
                &mut self,
                method: &str,
                temp_steps: u32,
                start_temp: f32,
                end_temp: f32,
                e_steps: u32,
                m_steps: u32,
            ) {
                let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
                let options = FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o755);

                zip.start_file("animation.gif", options)
                    .expect("error creating zip");
                zip.write_all(&self.encoder.get_ref())
                    .expect("error creating zip");

                zip.start_file("model_output.csv", options)
                    .expect("error creating zip");
                zip.write_all(
                    &self
                        .results
                        .make_csv_buffer()
                        .expect("creating csv should be infallable"),
                )
                .expect("error creating zip");

                zip.start_file("model_params.txt", options)
                    .expect("error creating zip");
                zip.write_all(
                    &self.make_model_params(method, temp_steps, start_temp, end_temp, e_steps, m_steps),
                )
                .expect("error creating zip");

                zip.start_file("distributions.json", options)
                    .expect("error creating zip");
                zip.write_all(
                    &self
                        .results
                        .make_json_buffer()
                        .expect("error creating distribution json"),
                )
                .expect("error creating zip");

                self.zip_data = Some(zip.finish().expect("error creating zip").into_inner());
            }

            fn make_model_params(
                &self,
                method: &str,
                temp_steps: u32,
                start_temp: f32,
                end_temp: f32,
                e_steps: u32,
                m_steps: u32,
            ) -> Vec<u8> {
                let mut buf = Vec::new();
                writeln!(buf, "Model Size: {}", $size).expect("error creating model parms file");
                writeln!(buf, "Method: {}", method).expect("error creating model parms file");
                writeln!(buf, "Energies: {}", self.system.get_energies_dict())
                    .expect("error creating model parms file");
                writeln!(
                    buf,
                    "Concentration A: {}, Concentration B: {}",
                    self.c_0, self.c_1
                )
                .expect("error creating model parms file");
                writeln!(buf, "Temperature Steps: {}", temp_steps)
                    .expect("error creating model parms file");
                writeln!(buf, "Start Temperature: {}, End Temperature: {}", start_temp, end_temp)
                    .expect("error creating model parms file");
                writeln!(buf, "Steps for Equilibrium: {}", e_steps)
                    .expect("error creating model parms file");
                writeln!(buf, "Steps for Measurement: {}", m_steps)
                    .expect("error creating model parms file");
                buf
            }

            pub fn get_zip_ptr(&self) -> *const u8 {
                self.zip_data
                    .as_ref()
                    .expect("zip data doesnt exist")
                    .as_ptr()
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

makemodel!(XSmallModel, 16, 4);
makemodel!(SmallModel, 32, 5);
makemodel!(MediumModel, 64, 6);
makemodel!(BigModel, 128, 7);
makemodel!(XBigModel, 256, 8);
