#[cfg(test)]
mod speed_tests {
    use crate::{math::Vector2, simulation_core, Particle, Sph, HEIGHT, SIM_CONF, WIDTH};
    use chrono::{Datelike, Timelike};
    use serde_derive::{Deserialize, Serialize};
    use std::{
        collections::LinkedList, fs::OpenOptions, io::Read, os::unix::fs::FileExt,
        process::Command, time::Instant,
    };

    const LOG_FILE: &str = "speed_test_results.json";

    #[derive(Serialize, Deserialize)]
    struct SpeedTestResult {
        commit_hash: String,
        commit_message: String,
        datetime: String,
        total_time_ms: u128,
        iterations: u32,
        total_avg_fps: f32,
        mean_time_diff_ms: u128,
        min_time_diff_ms: u128,
        max_time_diff_ms: u128,
        sph_particle_count: usize,
    }

    fn write_to_logs(mut test_result: SpeedTestResult) {
        let commit_msg_out = Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--pretty=%B")
            .output()
            .expect("Invalid command to get commit message!");
        let commit_hash_out = Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--pretty=%H")
            .output()
            .expect("Invalid command to get commit hash!");

        let msg = String::from_utf8(commit_msg_out.stdout)
            .unwrap()
            .trim()
            .to_owned();
        let hash = String::from_utf8(commit_hash_out.stdout)
            .unwrap()
            .trim()
            .to_owned();

        test_result.commit_hash = hash;
        test_result.commit_message = msg;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(LOG_FILE)
            .unwrap();
        let mut file_content = String::new();
        let _ = file.read_to_string(&mut file_content);

        let mut list: LinkedList<SpeedTestResult> = serde_json::from_str(&file_content).unwrap();
        list.push_front(test_result);
        let serialized = serde_json::to_string_pretty(&list).unwrap();
        let _ = file.write_all_at(serialized.as_bytes(), 0);
    }

    /// How long the test will run for.
    const TIME_SEC: u64 = 10;

    #[test]
    #[ignore]
    fn simulation_speed_test_over_time() {
        // TODO: FIX the test

        let mut sph = Sph::new(SIM_CONF, WIDTH, HEIGHT);
        // Add some particles
        let mut vec = Vec::with_capacity(1000);
        for i in (2..=(WIDTH as i32 - 2)).step_by(10) {
            for j in (2..=(HEIGHT as i32 - 300)).step_by(4) {
                vec.push(Particle::new(Vector2::new(i as f32, j as f32)));
            }
        }
        sph.particles = vec;

        // Metrics
        let mut iterations = 0u32;
        let mut time_diffs_ms = LinkedList::new();

        let mut now = Instant::now();
        let start_time = now;
        let start_datetime = chrono::Local::now();
        let mut last_time = start_time;
        let mut sec_since_start = 0;
        while sec_since_start < TIME_SEC {
            simulation_core(&mut sph);
            iterations += 1;

            now = Instant::now();
            let since_last = now.duration_since(last_time).as_millis();
            time_diffs_ms.push_back(since_last);

            sec_since_start = now.duration_since(start_time).as_secs();
            last_time = now;
        }
        let total_time = now.duration_since(start_time);

        let mut time_diffs_ms = time_diffs_ms.into_iter().collect::<Vec<_>>();
        time_diffs_ms.sort();

        let avg_fps = (iterations as f32) / total_time.as_secs_f32();
        let mean_diff = time_diffs_ms[time_diffs_ms.len() / 2];
        let min_diff = time_diffs_ms[0];
        let max_diff = time_diffs_ms[time_diffs_ms.len() - 1];

        let dt = start_datetime;
        let test_result = SpeedTestResult {
            commit_hash: String::new(),
            commit_message: String::new(),
            datetime: format!(
                "{:02}.{:02}.{} {:02}:{:02}:{:02}",
                dt.day(),
                dt.month(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            ),
            total_time_ms: total_time.as_millis(),
            iterations,
            total_avg_fps: avg_fps,
            mean_time_diff_ms: mean_diff,
            min_time_diff_ms: min_diff,
            max_time_diff_ms: max_diff,
            sph_particle_count: sph.particles.len(),
        };

        write_to_logs(test_result);
    }
}
