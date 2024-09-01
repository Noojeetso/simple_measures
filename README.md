simple_measures
=====
Крейт для простых измерений времени работы алгоритмов.  
Работает только на *nix системах.

### Использование

algorithms.rs
```rs
pub fn linear_algorithm(data: &mut Vec<f64>) -> f64 {
    let mut res = 0.0;
    for value in data.iter_mut() {
        *value *= 2.0;
        res += *value;
    }
    res
}

pub fn quadratic_algorithm(data: &Vec<f64>) -> f64 {
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    for i in 0..data.len() {
        sum += data[i];
        for j in 0..data.len() {
            sum2 += data[j];
        }
    }
    return sum + sum2;
}
```

generators.rs
```rs
use rand::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct VectorGenerator {
    seed: u64,
    rng: ChaCha8Rng,
}

impl VectorGenerator {
    pub fn new() -> Self {
        let time_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Ошибка при создании генератора")
            .as_millis();
        let seed: u64 = time_millis as u64;
        let rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(seed);
        Self { seed, rng }
    }

    pub fn generate_vector(&mut self, size: usize) -> Vec<f64> {
        let mut res = Vec::new();
        res.resize_with(size, || self.rng.gen::<f64>());
        res
    }

    pub fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
    }
}
```

main.rs
```rs
mod algorithms;
mod generators;

use algorithms::{linear_algorithm, quadratic_algorithm}; // Пользовательские алгоритмы, которые нужно измерить
use generators::VectorGenerator; // Определённая пользователем структура, хранящая состояние, для генерации данных, подаваемых на вход алгоритмам
use simple_measures::measures::{MeasurableAlgorithm, PackMeasures};

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

fn main() {
    // Первый измеряемый алгоритм
    let mut generator = example::generators::VectorGenerator::new();
    let generator_closure = |size: &usize| generator.generate_vector(*size);  // Замыкание, генерирующие данные для алгоритма
    let measurable_linear_algorithm = MeasurableAlgorithm::new_mut(  // метод new_mut, так как алгоритм изменяет свои аргументы
        "Линейный алгоритм",  // Название измеряемого алгоритма
        Box::new(linear_algorithm),  // Измеряемая функция
        Box::new(generator_closure),  // Замыкание-генератор
    )
    .with_filename("linear_algorithm");  // Имя файла, в который будут записываться результаты измерений. По умолчанию совпадает с названием алгоритма 

    // Второй измеряемый алгоритм
    let mut generator_2 = example::generators::VectorGenerator::new();
    let generator_closure_2 = |size: &usize| generator_2.generate_vector(*size);
    let measurable_quadratic_algorithm = MeasurableAlgorithm::new(  // метод new без mut, так как алгоритм не изменяет свои аргументы
        "Квадратичный алгоритм",
        Box::new(quadratic_algorithm),
        Box::new(generator_closure_2),
    )
    .with_filename("quadratic_algorithm");

    // Генерация вектора линейных размеров входных данных алгоритмов, который будет использоваться во время замеров
    let mut sizes = vec![];
    for i in 1..4 {
        for j in 1..10 {
            sizes.push(usize::pow(10, i) * j);
        }
    }
    // Набор измеряемых алгоритмов
    let mut pack_measures = PackMeasures::new(
            "Стандартный набор",  // Имя набора измеряемых функций
            sizes)
        .with_filename("default_pack")  // Название каталога, в который будут записаны файлы с результатами измерений, а также файл-описаниею. По умолчанию название совпадает с именем набора
        .with_threshold(Duration::new(1, 0))  // Ограничение на время выполнения алгоритмов. Максимальные размеры вычисляются единожды перед запуском первого измерения (при последующих запусках измерений размеры вычисляться не будут, но можно вручную вызвать соответствующую функцию)
        .with_iterations_amount(5)
        .with_x_label("Линейный размер данных")
        .with_y_label("Времени работы алгоритмов, мкс");
    pack_measures.add_target(measurable_linear_algorithm);
    pack_measures.add_target(measurable_quadratic_algorithm);
    pack_measures.use_threshold(true);  // Возможность отключить вычисление максимальных размеров перед измерениями. По умолчанию включено
    pack_measures.measure(5);  // Количество итераций работы алгоритма во время одного замера, в качестве результата замера берётся среднее значение времени работы
    pack_measures.write().unwrap();  // Запись результатов измерений на диск
    pack_measures.print();  // Вывод результатов измерений в виде таблицы в стандартный поток вывода

    // Построение графика
    match simple_measures::graph::graph::generate_single_graphic::<usize>(
        "default_pack",  // Имя каталога с результатами работы набора алгоритмов
        &PathBuf::from_str("default.conf").unwrap(),  // Путь к файлу с конфигурацией графика
    ) {
        Ok(()) => (),
        Err(e) => {
            eprint!("{:#}\n", e);
        }
    };
}
```

Файл конфигурации графика default.conf
```json
{
    "pack_name": "default_pack",
    "x_start" : 0,
    "x_end" : 0,
    "x_scale" : 1,
    "log_x" : false,
    "y_start" : 0,
    "y_end" : 0,
    "y_scale" : 1,
    "log_y" : false
}

```

График, полученный в результате измерений:  
![graph](./default_pack_graph.pdf)
