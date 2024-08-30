#[derive(Clone, Debug)]
pub struct Quartiles {
    bottom_boundary: f64,
    lower: f64,
    median: f64,
    upper: f64,
    top_boundary: f64,
}

impl Quartiles {
    pub fn new<T: Into<f64> + Copy + PartialOrd>(slice: &[T]) -> Self {
        let mut vector = slice.to_owned();
        vector.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        let bottom_boundary = (*vector.first().unwrap()).into();
        let top_boundary = (*vector.last().unwrap()).into();
        let lower = Quartiles::quantile(&vector, 25.0);
        let median = Quartiles::quantile(&vector, 50.0);
        let upper = Quartiles::quantile(&vector, 75.0);

        Self {
            bottom_boundary,
            lower,
            median,
            upper,
            top_boundary,
        }
    }

    pub fn values(&self) -> [f64; 5] {
        [
            self.bottom_boundary as f64,
            self.lower as f64,
            self.median as f64,
            self.upper as f64,
            self.top_boundary as f64,
        ]
    }

    fn quantile<T: Into<f64> + Copy>(slice: &[T], percent: f64) -> f64 {
        assert!(percent >= 0.0);
        assert!(percent <= 100.0);

        if slice.len() == 0 {
            return f64::NAN;
        }
        if slice.len() == 1 {
            return slice[0].into();
        }

        let max_index = slice.len() - 1;
        let raw_index = (percent / 100.0) * (max_index as f64);
        let bottom_index = raw_index.floor() as usize;

        if bottom_index == max_index {
            return slice[max_index].into();
        }
        let bottom_element = slice[bottom_index].into();
        let next_element = slice[bottom_index + 1].into();
        let distance = raw_index.fract();

        bottom_element + (next_element - bottom_element) * distance
    }
}
