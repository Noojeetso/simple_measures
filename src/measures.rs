use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;
use std::str::FromStr;
use std::time::Duration;
use std::time::SystemTime;
use std::{collections::HashMap, marker::PhantomData};

const DATA_DIR: &str = "data";

mod nix_function_threshold {
    use nix::errno::Errno;
    use nix::sys::signal::{kill, Signal};
    use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
    use nix::unistd::{fork, ForkResult};
    use std::process::exit;
    use std::time::Duration;

    pub unsafe fn call_long_running_function<ArgT, ResT, F: Fn(&ArgT) -> ResT>(
        function: &F,
        data: &ArgT,
        threshold: Duration,
    ) -> bool {
        let mut result = false;

        let child_pid = match fork() {
            Ok(ForkResult::Child) => {
                _ = function(data);
                exit(0);
            }

            Ok(ForkResult::Parent { child, .. }) => {
                // println!(
                //     "[call_long_running_function] forked a child with PID {}.",
                //     child
                // );
                child
            }

            Err(err) => {
                panic!("[call_long_running_function] fork() failed: {}", err);
            }
        };

        // println!("Child pid: {}", child_pid);

        let time = std::time::Instant::now();
        loop {
            std::thread::sleep(std::time::Duration::from_secs_f64(0.1));
            match waitpid(child_pid, Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::StillAlive) => {
                    // println!(
                    //     "[call_long_running_function] Child is still alive, do my own stuff while waiting."
                    // );
                }

                Ok(_status) => {
                    // println!(
                    //     "[call_long_running_function] Child exited with status {:?}.",
                    //     status
                    // );
                    result = true;
                    break;
                }

                Err(err) => panic!("[call_long_running_function] waitpid() failed: {}", err),
            }
            let took = time.elapsed();
            if took > threshold {
                match kill(child_pid, Signal::SIGKILL) {
                    Ok(_) => {
                        // println!("Sent termination signal to the child process");
                    }
                    Err(err) => {
                        if err != Errno::ESRCH {
                            eprintln!(
                                "[call_long_running_function] Error sending termination signal: {}",
                                err
                            );
                        } else {
                            // println!(
                            //     "We tried to send a signal, but process has already been terminated"
                            // )
                        }
                    }
                }
                break;
            }
        }
        // println!("result: {}", result);

        return result;
    }
}

pub struct MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display,
{
    pub name: String,
    pub algorithm: Box<dyn Fn(&AlgArgT) -> AlgResT + 'a>,
    pub generator: RefCell<Box<dyn FnMut(&GenArgT) -> AlgArgT + 'a>>,
    gen_art: PhantomData<GenArgT>,
    alg_art: PhantomData<AlgArgT>,
    alg_res: PhantomData<AlgResT>,
}

impl<'a, GenArgT, AlgArgT, AlgResT> MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display,
{
    pub fn new(
        name: &str,
        algorithm: Box<dyn Fn(&AlgArgT) -> AlgResT + 'a>,
        generator: Box<dyn FnMut(&GenArgT) -> AlgArgT + 'a>,
    ) -> Self {
        MeasurableAlgorithm {
            name: name.to_string(),
            algorithm,
            generator: RefCell::new(generator),
            gen_art: PhantomData,
            alg_art: PhantomData,
            alg_res: PhantomData,
        }
    }

    fn measure(&self, sizes: &[GenArgT], iterations_amount: u64) -> Vec<Duration> {
        let mut elapsed_time_for_sizes: Vec<Duration> = Vec::new();

        let mut generator = self.generator.borrow_mut();

        for size in sizes {
            let mut current_elapsed_time = Duration::new(0, 0);

            let data: AlgArgT = (generator.deref_mut())(size);

            let stopwatch: SystemTime = SystemTime::now();
            // let stopwatch = ProcessTime::now();

            for _ in 0..iterations_amount {
                (self.algorithm)(&data);
            }
            current_elapsed_time += stopwatch
                .elapsed()
                .expect("Ошибка определения wall-clock времени работы алгоритма");
            // current_duration += stopwatch
            //     .try_elapsed()
            //     .expect("Ошибка определения процессорного времени в конце работы");

            current_elapsed_time = current_elapsed_time
                .checked_div(iterations_amount as u32)
                .expect("Ошибка: количество повторов измерения равно нулю");
            elapsed_time_for_sizes.push(current_elapsed_time);
        }

        elapsed_time_for_sizes
    }

    fn calculate_max_data_size(&self, sizes: &[GenArgT], threshold: Duration) -> usize {
        use std::io::{stdout, Write};
        let mut max_size_index: usize = 0;
        let mut generator = self.generator.borrow_mut();
        let mut lock = stdout().lock();
        unsafe {
            println!("Алгоритм: {}", self.name);
            for size in sizes {
                write!(lock, "Линейный размер входных данных: {}\t\r", size).unwrap();
                _ = std::io::stdout().flush();
                let data = (generator.deref_mut())(size);
                let res = nix_function_threshold::call_long_running_function(
                    &self.algorithm,
                    &data,
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

impl<'a, GenArgT, AlgArgT, AlgResT> PartialEq for MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display,
{
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a, GenArgT, AlgArgT, AlgResT> Eq for MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT> where
    GenArgT: std::fmt::Display
{
}

impl<'a, GenArgT, AlgArgT, AlgResT> Hash for MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub struct AlgorithmTimeStatistic {
    pub max_size_number: usize,
    pub measures: Vec<Vec<Duration>>,
}

pub struct PackMeasures<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display,
{
    name: String,
    sizes: Vec<GenArgT>,
    x_label: String,
    y_label: String,
    iterations_amount: u64,
    threshold: Duration,
    time_statistics:
        HashMap<MeasurableAlgorithm<'a, GenArgT, AlgArgT, AlgResT>, AlgorithmTimeStatistic>,
    need_max_sizes_update: bool,
}

impl<'a, GenArgT, AlgArgT, AlgResT> PackMeasures<'a, GenArgT, AlgArgT, AlgResT>
where
    GenArgT: std::fmt::Display,
{
    pub fn new(name: &str, sizes: Vec<GenArgT>) -> Self {
        PackMeasures {
            name: name.to_string(),
            sizes,
            x_label: String::from_str("Аргументы функций").unwrap(),
            y_label: String::from_str("Значения функций").unwrap(),
            iterations_amount: 5,
            threshold: Duration::new(1, 0),
            time_statistics: HashMap::new(),
            need_max_sizes_update: true,
        }
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

    pub fn measure(&mut self, measures_amount: u64) {
        use std::io::{stdout, Write};
        if self.need_max_sizes_update {
            self.calculate_max_data_sizes();
            self.need_max_sizes_update = false;
        }
        println!("Замер времени выполнения ({})", self.name);
        let time = std::time::Instant::now();
        for (algorithm, statistic) in self.time_statistics.iter_mut() {
            println!("Алгоритм: {}", algorithm.name);
            let mut lock = stdout().lock();
            for i in 0..measures_amount {
                write!(lock, "Номер замера: {}/{}\t\r", i + 1, measures_amount).unwrap();
                _ = std::io::stdout().flush();
                let elapsed_time_for_sizes = algorithm.measure(
                    &self.sizes[0..statistic.max_size_number],
                    self.iterations_amount,
                );
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
        println!("Расчёт максимальных размеров");
        let time = std::time::Instant::now();
        for (algorithm, statistics) in self.time_statistics.iter_mut() {
            statistics.max_size_number =
                algorithm.calculate_max_data_size(&self.sizes, self.threshold);
        }
        let took = time.elapsed();
        println!("Расчёт занял {:.3}с\n", took.as_secs_f64());
        for statistics in self.time_statistics.values_mut() {
            statistics
                .measures
                .resize(statistics.max_size_number, vec![]);
        }
    }

    pub fn write(&self) {
        use std::io::Write;
        for (algorithm, statistic) in self.time_statistics.iter() {
            let relative_path = format!("{}/{}/{}/", DATA_DIR, self.name, algorithm.name);
            std::fs::create_dir_all(&relative_path)
                .expect(format!("Не удалось создать каталог {}", relative_path).as_str());
            for i in 0..statistic.max_size_number {
                let file_path = format!("{}{}.txt", relative_path, self.sizes[i]);
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)
                    .expect("Не удалось создать файл");

                let res_str = statistic.measures[i]
                    .iter()
                    .fold(String::new(), |mut a, b| {
                        a.push_str(&b.as_micros().to_string());
                        a.push('\n');
                        a
                    });

                if let Err(e) = file.write(res_str.as_bytes()) {
                    eprintln!("Ошибка при записи в файл: {}", e);
                };
            }
        }
    }

    pub fn print(&self) {
        use prettytable::{format::Alignment, Cell, Row, Table};
        let mut table = Table::new();
        let mut cells: Vec<Cell> = Vec::new();
        cells.push(Cell::new(&self.x_label));
        for algorithm in self.time_statistics.keys() {
            cells.push(Cell::new(algorithm.name.as_str()));
        }
        table.add_row(Row::new(vec![Cell::new_align(
            self.name.as_str(),
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
