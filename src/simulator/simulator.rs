pub struct Input(pub(super) Vec<u32>);

pub struct Output(pub(super) Vec<u32>);

#[derive(Clone, Debug)]
pub struct Gate {
    pub id: u32,
    pub a: u32,
    pub b: u32,
    pub meta: Option<Box<GateMeta>>,
}

impl Gate {
    pub fn meta(&self) -> Option<&GateMeta> {
        self.meta.as_deref()
    }

    pub fn add_meta(&mut self) -> &mut GateMeta {
        self.meta.get_or_insert_with(|| Default::default())
    }

    pub fn is_input(&self) -> bool {
        self.meta().map(|m| m.input_id.is_some()).unwrap_or(false)
    }

    /*pub fn is_output(&self) -> bool {
        self.meta().map(|m| m.output_id.is_some()).unwrap_or(false)
    }*/

    pub fn is_io(&self) -> bool {
        self.meta().map(|m| m.input_id.is_some() || m.output_id.is_some()).unwrap_or(false)
    }

    pub fn is_pinned(&self) -> bool {
        self.meta().map(|m| m.output_id.is_some() || m.input_id.is_some() || m.pinned || !m.names.is_empty()).unwrap_or(false)
    }
}

#[derive(Clone, Debug, Default)]
pub struct GateMeta {
    pub pinned: bool,
    pub names: Vec<String>,
    pub input_id: Option<u32>,
    pub output_id: Option<u32>,
}

pub trait Simulator {
    fn new(gates: &[Gate]) -> Self;

    fn set(&mut self, input: &Input, bits: impl Into<u64>);
    fn get<R: TryFrom<u64>>(&self, output: &Output) -> R
        where <R as TryFrom<u64>>::Error: std::fmt::Debug;

    /// Runs the simulation for one timestep
    fn step(&mut self);

    /// Runs the simulation for n timesteps
    fn step_by(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// Runs the simulation until it settles or a maximum numbe of timesteps. Returns the number of
    /// steps if the simulation settled within the allotted number of steps, or None if it didn't.
    fn step_until_settled(&mut self, max_steps: usize) -> Option<usize>;

    fn snapshot(&mut self);

    fn show(&self);

    fn clear(&mut self);

    fn num_gates(&self) -> usize;
}
