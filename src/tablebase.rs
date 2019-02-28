use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::BufReader;
use std::io::Read;

pub struct Tablebase {
    pub dp: Vec<u8>
}

impl Tablebase {
    pub fn write_to_disk(&self, file: File) {
        let mut writer = BufWriter::new(&file);
        for r in &self.dp {
            writer.write(&[*r]);
        }
    }

    pub fn read_from_disk(file: File) -> Self {
        let reader = BufReader::new(&file);
        let mut dp = Vec::new();
        for r in reader.bytes() {
            dp.push(r.expect("wat"))
        }
        Tablebase{dp}
    }

    pub fn print_stats(&self) {
        let mut mate = 0;
        let mut draw = 0;
        let mut not_calculated = 0;
        for s in &self.dp {
            match s {
                254 => draw += 1,
                255 => not_calculated += 1,
                _ => mate += 1
            }
        }
        let percent = 100.0 / self.dp.len() as f32;
        println!("{} ({}%) mate, {} ({}%) draw, {} ({}%) not calculated from {} total", mate, mate as f32 * percent, draw, draw as f32  * percent, not_calculated, not_calculated as f32  * percent, self.dp.len())
    }
}