use error_stack::Report;

trait Sealed {
    fn get_sealed_value(self) -> bool;
}

#[allow(private_bounds)]
pub trait BoolHelper: Sized + Sealed {
    fn then_err<F, E>(self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> E,
        E: std::error::Error + Send + Sync + 'static,
    {
        if self.get_sealed_value() {
            Err(f())
        } else {
            Ok(())
        }
    }

    fn then_err_report<F, E>(self, f: F) -> Result<(), Report<E>>
    where
        F: FnOnce() -> E,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.then_err(f).map_err(Report::new)
    }
}

impl Sealed for bool {
    fn get_sealed_value(self) -> bool {
        self
    }
}

impl BoolHelper for bool {}

#[cfg(test)]
mod tests {
    use super::*;
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("test error")]
    struct TestError;

    #[test]
    fn then_err() {
        assert!(false.then_err(|| TestError).is_ok());
        assert!(true.then_err(|| TestError).is_err());
    }
}
