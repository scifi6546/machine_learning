pub use chrono;
use chrono::{DateTime, TimeDelta, Utc};
pub use rusqlite;
pub use thiserror;
#[derive(Debug, Clone, Copy)]
pub struct TestAssertionOptions {
    max_delta: TimeDelta,
}
impl Default for TestAssertionOptions {
    fn default() -> Self {
        Self {
            max_delta: TimeDelta::milliseconds(1),
        }
    }
}
impl TestAssertionOptions {
    pub fn max_difference(mut self, max_delta: TimeDelta) -> Self {
        self.max_delta = max_delta;
        self
    }
}
#[derive(Debug)]
pub struct TestAssertion {
    value: DateTime<Utc>,
    label: Option<String>,
    options: TestAssertionOptions,
}
impl TestAssertion {
    pub fn to_be_close_to(&self, value: DateTime<Utc>) {
        let real_difference = (self.value - value).abs();
        if real_difference > self.options.max_delta {
            if let Some(label) = &self.label {
                panic!(
                    "{}\n{} - {} > {}\n{} - {} = {}",
                    label,
                    self.value,
                    value,
                    self.options.max_delta,
                    self.value,
                    value,
                    real_difference
                )
            } else {
                panic!(
                    "{} - {} > {}\n{} - {} = {}",
                    self.value, value, self.options.max_delta, self.value, value, real_difference
                );
            }
        }
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn with_options(mut self, options: TestAssertionOptions) -> Self {
        self.options = options;
        self
    }
}

pub fn expect(value: DateTime<Utc>) -> TestAssertion {
    TestAssertion {
        value,
        label: None,
        options: TestAssertionOptions::default(),
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_assertion_close_to() {
        expect("2010-02-27T06:30:00.000Z".parse().unwrap())
            .to_be_close_to("2010-02-27T06:30:00.000Z".parse().unwrap());
        expect("2010-02-27T06:30:00.000Z".parse().unwrap())
            .to_be_close_to("2010-02-27T06:30:00.001Z".parse().unwrap());
    }
    #[test]
    fn test_assertion_options() {
        expect("2010-02-27T06:30:00.000Z".parse().unwrap())
            .with_options(TestAssertionOptions::default().max_difference(TimeDelta::hours(1)))
            .to_be_close_to("2010-02-27T06:40:00.000Z".parse().unwrap());
    }
    #[test]
    #[should_panic]
    fn test_assertion_options_fail() {
        expect("2010-02-27T06:30:00.000Z".parse().unwrap())
            .with_options(TestAssertionOptions::default().max_difference(TimeDelta::hours(1)))
            .to_be_close_to("2010-02-27T08:40:00.000Z".parse().unwrap());
    }
    #[test]
    #[should_panic]
    fn assertion_far() {
        expect("2010-02-27T06:30:00.000Z".parse().unwrap())
            .to_be_close_to("2011-02-27T06:30:00.000Z".parse().unwrap());
    }
}
