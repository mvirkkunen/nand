

    pub fn show(&self) {
        let pad = self.names.iter().map(|(_, name, _)| name.len()).max().unwrap() + 1;

        for (_, name, out) in &self.names {
            println!("{name:pad$}{out}", name=name, pad=pad, out=out);
        }

        println!("max steps: {}", self.max_steps);
        println!("gates: {}", self.gates.len());
    }

    pub fn bench(&mut self) {
        let steps: u64 = 100_000;

        let start = SystemTime::now();
        for _ in 0..steps {
            self.step();
        }
        let end = SystemTime::now();

        println!(
            "steps per second: {}",
            steps * 1_000_000 / (end.duration_since(start).unwrap().as_micros() as u64));
    }