pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct GraphError {
    pub repr: GraphErrorRepr,
}

#[derive(Debug, Clone)]
pub enum GraphErrorRepr {
    DataPreprocessingError,
    ParseError,
    UTF8Error,
}

#[derive(Debug, Clone)]
pub struct DataPreprocessingError;
impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.repr {
            GraphErrorRepr::DataPreprocessingError => {
                write!(
                    f,
                    "Ошибка, связанная с предварительной подготовкой файлов данных"
                )
            }
            GraphErrorRepr::ParseError => {
                write!(f, "Ошибка, связанная с размером матрицы")
            }
            GraphErrorRepr::UTF8Error => {
                write!(f, "Ошибка, связанная с utf-8 кодированием")
            }
        }
    }
}

// impl GraphError {
//     fn kind(&self) -> GraphErrorKind {
//         self.error_kind.clone()
//     }
// }

impl std::error::Error for GraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.repr {
            GraphErrorRepr::DataPreprocessingError => None,
            GraphErrorRepr::ParseError => None,
            GraphErrorRepr::UTF8Error => None,
        }
    }
}

// impl std::error::Error for GraphError {
//     #[allow(deprecated, deprecated_in_future)]
//     fn description(&self) -> &str {
//         match self.repr {
//             GraphErrorRepr::Os(..) | Repr::Simple(..) => self.kind().as_str(),
//             GraphErrorRepr::Custom(ref c) => c.error.description(),
//         }
//     }

//     #[allow(deprecated)]
//     fn cause(&self) -> Option<&dyn error::Error> {
//         match self.repr {
//             Repr::Os(..) => None,
//             Repr::Simple(..) => None,
//             Repr::Custom(ref c) => c.error.cause(),
//         }
//     }

//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         match self.repr {
//             Repr::Os(..) => None,
//             Repr::Simple(..) => None,
//             Repr::Custom(ref c) => c.error.source(),
//         }
//     }
// }
