use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::BufReader;
use std::io::Read;
use crate::search::retrograde_search;
use rayon::ThreadPoolBuilder;
use crate::verification::verify;
use rayon::ThreadPool;
use indicatif::ProgressBar;

pub struct Tablebase {
    pub dp: Vec<u8>
}

impl Tablebase {
    pub fn write_to_disk(&self, file: File) {
        println!("Writing tablebase to disk...");
        let mut writer = BufWriter::new(&file);
        let mut i = 0;
        let bar = ProgressBar::new(self.dp.len()as u64);
        for r in &self.dp {
            writer.write(&[*r]);
            i += 1;
            if i == 20 {
                bar.inc(20);
                i = 0;
            }
        }
        bar.finish();
    }

    pub fn read_from_disk(file: File) -> Self {
        println!("Reading tablebase from disk...");
        let reader = BufReader::new(&file);
        let mut dp = Vec::new();
        let bar = ProgressBar::new(file.metadata().unwrap().len());
        let mut i = 0;
        for r in reader.bytes() {
            dp.push(r.unwrap());
            i += 1;
            if i == 20 {
                bar.inc(20);
                 i = 0;
            }
        }
        bar.finish();
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

    pub fn generate(threads: usize) -> Self {
        retrograde_search(ThreadPoolBuilder::new().num_threads(threads).build().unwrap())
    }

    pub fn verify(&self, threads: usize) -> bool {
        verify(&self.dp, ThreadPoolBuilder::new().num_threads(threads).build().unwrap())
    }
}