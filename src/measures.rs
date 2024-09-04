use crate::description;

use cpu_time::{ProcessTime, ThreadTime};
use fs_err as fs;

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;
use std::time::Duration;

const PACKS_DIR: &str = "packs";
const DATA_DIR: &str = "data";

pub enum TimerType {
    ProcessTimer,
    ThreadTimer,
    SystemTimer,
}

pub trait Timer {
    fn now() -> Self;
    fn elapsed(&self) -> Duration;
}

impl Timer for ProcessTime {
    fn now() -> Self {
        Self::now()
    }
    fn elapsed(&self) -> Duration {
        self.elapsed()
    }
}

impl Timer for ThreadTime {
    fn now() -> Self {
        Self::now()
    }
    fn elapsed(&self) -> Duration {
        self.elapsed()
    }
}

impl Timer for SystemTime {
    fn now() -> Self {
        Self::now()
    }
    fn elapsed(&self) -> Duration {
        self.elapsed().unwrap()
    }
}

pub enum Algorithm<'a, AlgArgT, AlgResT> {
    NonMutatingAlgorithm(Box<dyn Fn(&AlgArgT) -> AlgResT + 'a>),
    MutatingAlgorithm(Box<dyn Fn(&mut AlgArgT) -> AlgResT + 'a>),
}

pub struct MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT> {
    pub filename: String,
    pub description: String,
    pub algorithm: Algorithm<'a, AlgArgT, AlgResT>,
    pub generator: RefCell<Box<dyn FnMut(&GenArgT) -> AlgArgT + 'a>>,
    current_data: Option<AlgArgT>,
    gen_arg: PhantomData<GenArgT>,
    alg_arg: PhantomData<AlgArgT>,
    alg_res: PhantomData<AlgResT>,
}

impl<'a, GenArgT, AlgArgT, AlgResT> MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT> {
    pub fn new(
        description: &str,
        algorithm: Box<dyn Fn(&AlgArgT) -> AlgResT + 'a>,
        generator: Box<dyn FnMut(&GenArgT) -> AlgArgT + 'a>,
    ) -> Self {
        Self {
            description: description.to_string(),
            filename: description.to_string(),
            algorithm: Algorithm::NonMutatingAlgorithm(algorithm),
            generator: RefCell::new(generator),
            current_data: None,
            gen_arg: PhantomData,
            alg_arg: PhantomData,
            alg_res: PhantomData,
        }
    }

    pub fn new_mut(
        description: &str,
        algorithm: Box<dyn Fn(&mut AlgArgT) -> AlgResT + 'a>,
        generator: Box<dyn FnMut(&GenArgT) -> AlgArgT + 'a>,
    ) -> Self {
        Self {
            description: description.to_string(),
            filename: description.to_string(),
            algorithm: Algorithm::MutatingAlgorithm(algorithm),
            generator: RefCell::new(generator),
            current_data: None,
            gen_arg: PhantomData,
            alg_arg: PhantomData,
            alg_res: PhantomData,
        }
    }

    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_string();
        self
    }

    pub fn set_current_data(&mut self, data: AlgArgT) {
        self.current_data = Some(data);
    }

    fn measure<TimerT>(&self, sizes: &[GenArgT], iterations_amount: u64) -> Vec<Duration>
    where
        TimerT: Timer
        {
        let mut elapsed_time_for_sizes: Vec<Duration> = Vec::new();

        let mut generator = self.generator.borrow_mut();

        for size in sizes.iter() {
            let mut current_elapsed_time = Duration::new(0, 0);

            let mut data = vec![];
            for _ in 0..iterations_amount {
                data.push((generator.deref_mut())(size));
            }

            let stopwatch = TimerT::now();

            match &self.algorithm {
                Algorithm::NonMutatingAlgorithm(algorithm) => {
                    for _ in 0..iterations_amount as usize {
                        let curr_data = data.pop().unwrap();
                        _ = algorithm(&curr_data);
                    }
                }
                Algorithm::MutatingAlgorithm(algorithm) => {
                    for _ in 0..iterations_amount as usize {
                        let mut curr_data = data.pop().unwrap();
                        _ = algorithm(&mut curr_data);
                    }
                }
            }

            current_elapsed_time += stopwatch.elapsed();

            current_elapsed_time = current_elapsed_time
                .checked_div(iterations_amount as u32)
                .expect("Ошибка: количество повторов измерения равно нулю");
            elapsed_time_for_sizes.push(current_elapsed_time);
        }

        elapsed_time_for_sizes
    }
}

impl<'a, GenArgT, AlgArgT, AlgResT> MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display,
{
    fn calculate_max_data_size(&self, sizes: &[GenArgT], threshold: Duration) -> usize {
        use crate::nix_function_threshold;
        use std::io::{stdout, Write};
        let mut max_size_index: usize = 0;
        let mut generator = self.generator.borrow_mut();
        let mut lock = stdout().lock();
        unsafe {
            println!("Алгоритм: {}", self.description);
            for size in sizes {
                write!(
                    lock,
                    "Максимальный линейный размер входных данных: {}\t\r",
                    size
                )
                .unwrap();
                _ = std::io::stdout().flush();
                let data = (generator.deref_mut())(size);
                let res = nix_function_threshold::call_long_running_function(
                    &self.algorithm,
                    data,
                    threshold,
                );
                if res == false {
                    break;
                }
                max_size_index += 1;
            }
            println!();
        }
        max_size_index
    }
}

impl<'a, GenArgT, AlgArgT, AlgResT> PartialEq
    for MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>
{
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
    }
}

impl<'a, GenArgT, AlgArgT, AlgResT> Eq for MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT> {}

impl<'a, GenArgT, AlgArgT, AlgResT> Hash for MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
    }
}

pub struct AlgorithmTimeStatistic {
    pub max_size_number: usize,
    pub measures: Vec<Vec<Duration>>,
}

pub struct PackMeasures<'a, GenArgT, AlgArgT, AlgResT> {
    description: String,
    filename: String,
    sizes: Vec<GenArgT>,
    timer: TimerType,
    x_label: String,
    y_label: String,
    iterations_amount: u64,
    use_threshold: bool,
    threshold: Duration,
    time_statistics:
        HashMap<MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>, AlgorithmTimeStatistic>,
    need_max_sizes_update: bool,
}

impl<'a, GenArgT, AlgArgT, AlgResT> PackMeasures<'a, GenArgT, AlgArgT, AlgResT> {
    pub fn new(name: &str, sizes: Vec<GenArgT>) -> Self {
        PackMeasures {
            description: name.to_string(),
            filename: name.to_string(),
            sizes,
            timer: TimerType::ProcessTimer,
            x_label: String::from_str("Аргументы функций").unwrap(),
            y_label: String::from_str("Значения функций").unwrap(),
            iterations_amount: 5,
            use_threshold: false,
            threshold: Duration::new(1, 0),
            time_statistics: HashMap::new(),
            need_max_sizes_update: true,
        }
    }

    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_string();
        self
    }

    pub fn with_timer(mut self, timer: TimerType) -> Self {
        self.timer = timer;
        self
    }

    pub fn with_x_label(mut self, x_label: &str) -> Self {
        self.x_label = x_label.to_string();
        self
    }

    pub fn with_y_label(mut self, y_label: &str) -> Self {
        self.y_label = y_label.to_string();
        self
    }

    pub fn with_iterations_amount(mut self, iterations_amount: u64) -> Self {
        self.iterations_amount = iterations_amount;
        self
    }

    pub fn with_threshold(mut self, threshold: Duration) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn set_threshold(&mut self, threshold: Duration) {
        self.threshold = threshold;
    }

    pub fn use_threshold(&mut self, condition: bool) {
        self.use_threshold = condition;
    }

    pub fn add_target(
        &mut self,
        measurable_algorithm: MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>,
    ) {
        self.time_statistics.insert(
            measurable_algorithm,
            AlgorithmTimeStatistic {
                max_size_number: 0,
                measures: vec![],
            },
        );
    }
}

impl<'a, GenArgT, AlgArgT, AlgResT> PackMeasures<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display + Clone,
{
    pub fn measure(&mut self, measures_amount: u64) {
        use std::io::{stdout, Write};
        if self.need_max_sizes_update {
            self.calculate_max_data_sizes();
            self.need_max_sizes_update = false;
        }
        println!("Замер времени выполнения ({})", self.description);
        let time = std::time::Instant::now();
        for (algorithm, statistic) in self.time_statistics.iter_mut() {
            println!("Алгоритм: {}", algorithm.description);
            let mut lock = stdout().lock();
            for i in 0..measures_amount {
                write!(lock, "Номер замера: {}/{}\t\r", i + 1, measures_amount).unwrap();
                _ = std::io::stdout().flush();
                let measure_sizes = &self.sizes[0..statistic.max_size_number];
                let elapsed_time_for_sizes = match &self.timer {
                    TimerType::ProcessTimer => algorithm.measure::<ProcessTime>(
                        measure_sizes,
                        self.iterations_amount
                    ),
                    TimerType::ThreadTimer => algorithm.measure::<ThreadTime>(
                        measure_sizes,
                        self.iterations_amount
                    ),
                    TimerType::SystemTimer => algorithm.measure::<SystemTime>(
                        measure_sizes,
                        self.iterations_amount
                    ),
                };
                for (i, time_elapsed) in elapsed_time_for_sizes.into_iter().enumerate() {
                    statistic.measures[i].push(time_elapsed);
                }
            }
            println!();
        }
        let took = time.elapsed();
        println!("Замер занял {:.3}с\n", took.as_secs_f64());
    }

    pub fn calculate_max_data_sizes(&mut self) {
        if self.use_threshold {
            println!("Расчёт максимальных размеров");
            let time = std::time::Instant::now();
            for (algorithm, statistics) in self.time_statistics.iter_mut() {
                statistics.max_size_number =
                    algorithm.calculate_max_data_size(&self.sizes, self.threshold);
            }
            let took = time.elapsed();
            println!("Расчёт занял {:.3}с\n", took.as_secs_f64());
        } else {
            for (_, statistics) in self.time_statistics.iter_mut() {
                statistics.max_size_number = self.sizes.len();
            }
        }
        for statistics in self.time_statistics.values_mut() {
            statistics
                .measures
                .resize(statistics.max_size_number, vec![]);
        }
    }
}

impl<'a, GenArgT, AlgArgT, AlgResT> PackMeasures<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display + Clone + serde::ser::Serialize,
{
    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
        let data_path =
            PathBuf::from_str(format!("{}/{}/{}", PACKS_DIR, self.filename, DATA_DIR).as_str())?;
        if !data_path.is_dir() {
            fs::create_dir_all(data_path)?;
        }
        let mut descriptions = vec![];
        for (algorithm, statistic) in self.time_statistics.iter() {
            let target_description = description::TargetDescription {
                filename: algorithm.filename.clone(),
                description: algorithm.description.clone(),
                max_size_number: statistic.max_size_number,
            };
            descriptions.push(target_description);
        }
        let pack_description = description::PackMeasuresDescription {
            filename: self.filename.clone(),
            description: self.description.clone(),
            sizes: self.sizes.clone(),
            x_label: self.x_label.clone(),
            y_label: self.y_label.clone(),
            iterations_amount: self.iterations_amount,
            threshold: self.threshold,
            target_descriptions: descriptions,
        };
        let pack_description_dir_path =
            PathBuf::from_str(format!("{}/{}/", PACKS_DIR, self.filename).as_str())?;
        // println!(
        //     "path: {}\n",
        //     pack_description_file_path.as_os_str().to_str().unwrap()
        // );
        pack_description.write(&pack_description_dir_path)?;
        for (algorithm, statistic) in self.time_statistics.iter() {
            let relative_path = format!(
                "{}/{}/{}/{}/",
                PACKS_DIR, self.filename, DATA_DIR, algorithm.filename
            );
            fs::create_dir_all(&relative_path)
                .expect(format!("Не удалось создать каталог {}", relative_path).as_str());
            for i in 0..statistic.max_size_number {
                let file_path = format!("{}{}.txt", relative_path, self.sizes[i]);
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)?;

                let res_str = statistic.measures[i]
                    .iter()
                    .fold(String::new(), |mut a, b| {
                        a.push_str(&b.as_micros().to_string());
                        a.push('\n');
                        a
                    });

                if let Err(e) = file.write(res_str.as_bytes()) {
                    eprintln!("Ошибка при записи в файл: {}", e);
                    return Err(e.into());
                };
            }
        }
        Ok(())
    }

    pub fn print(&self) {
        use prettytable::{format::Alignment, Cell, Row, Table};
        let mut table = Table::new();
        let mut cells: Vec<Cell> = Vec::new();
        cells.push(Cell::new(&self.x_label));
        for algorithm in self.time_statistics.keys() {
            cells.push(Cell::new(&algorithm.description.as_str()));
        }
        table.add_row(Row::new(vec![Cell::new_align(
            self.filename.as_str(),
            Alignment::CENTER,
        )
        .with_hspan(self.time_statistics.keys().len() + 1)]));

        table.add_row(Row::new(cells));
        for i in 0..self.sizes.len() {
            let mut cells: Vec<Cell> = Vec::new();
            cells.push(Cell::new(self.sizes[i].to_string().as_str()));
            for statistic in self.time_statistics.values() {
                let time_str: String = if i < statistic.max_size_number {
                    let mean_time_elapsed = statistic.measures[i]
                        .iter()
                        .fold(Duration::new(0, 0), |acc, e| acc + *e)
                        .checked_div(statistic.measures[i].len() as u32)
                        .unwrap();
                    format!("{}", mean_time_elapsed.as_nanos())
                } else {
                    format!(">{}", self.threshold.as_nanos())
                };

                cells.push(Cell::new(&time_str));
            }
            table.add_row(Row::new(cells));
        }

        println!("{}", self.y_label);
        table.printstd();
    }
}
