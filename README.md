simple_measures
=====
Крейт для простых измерений времени работы алгоритмов.  
Работает только на *nix системах.

### Использование

```rs
mod algorithms;

use algorithms::algorithms::{solve_sle_cramer, solve_sle_cramer_parallel}; // Пользовательские алгоритмы, которые нужно измерить
use algorithms::generators::MatrixGenerator; // Определённая пользователем структура, хранящая состояние, для генерации данных, подаваемых на вход алгоритмам
use simple_measures::measures::{MeasurableAlgorithm, PackMeasures};
use std::time::Duration;

fn main() {
    // Первый измеряемый алгоритм
    let mut generator = MatrixGenerator::new();
    let generator_closure = |side_size: &usize| generator.generate_sle_matrix(side_size);  // Замыкание, генерирующие данные для алгоритма
    let measurable_solve_sle_cramer = MeasurableAlgorithm::new(
        "Метод Крамера",  // Название измеряемого алгоритма
        Box::new(solve_sle_cramer),  // Измеряемая функция
        Box::new(generator_closure),  // Замыкание-генератор
    );

    // Второй измеряемый алгоритм
    let mut generator_2 = algorithms::generators::MatrixGenerator::new();
    let generator_function_2 =
        Box::new(|side_size: &usize| generator_2.generate_sle_matrix(side_size));
    let measurable_solve_sle_cramer_parallel = MeasurableAlgorithm::new(
        "Метод Крамера (параллельный)",
        Box::new(solve_sle_cramer_parallel),
        Box::new(generator_function_2),
    );

    // Набор для измерений
    let mut pack_measures =
        PackMeasures::new("Стандартный набор",  // Имя набора измеряемых функций
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])  // Линейные размеры данных, которые будут произведены замыканиями-генераторами и затем переданы на вход тестируемым алгоритмам
            .with_threshold(Duration::new(1, 0))
            .with_iterations_amount(5)
            .with_x_label("Количество уравнений в СЛАУ")
            .with_y_label(
                "Зависимость времени работы алгоритмов в микросекундах от размера входных данных",
            );
    pack_measures.add_target(measurable_solve_sle_cramer);  // Добавление целевого алгоритма в набор измеряемых функций
    pack_measures.add_target(measurable_solve_sle_cramer_parallel);
    pack_measures.measure(5);  // Количество измерений
    pack_measures.write();  // Запись измерений в файл ~/data/{algo-name}/{data-linear-size}.txt
    pack_measures.print();  // Вывод таблицы в стандартный поток
}
```