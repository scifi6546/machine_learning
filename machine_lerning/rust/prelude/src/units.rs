pub type FloatingPoint = f64;
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Meters(pub FloatingPoint);
impl From<FloatingPoint> for Meters {
    fn from(value: FloatingPoint) -> Self {
        Self(value)
    }
}
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Latitude(pub FloatingPoint);
impl From<FloatingPoint> for Latitude {
    fn from(value: FloatingPoint) -> Self {
        Self(value)
    }
}
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Longitude(pub FloatingPoint);
impl From<FloatingPoint> for Longitude {
    fn from(value: FloatingPoint) -> Self {
        Self(value)
    }
}
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Hertz(pub FloatingPoint);
impl From<FloatingPoint> for Hertz {
    fn from(value: FloatingPoint) -> Self {
        Self(value)
    }
}
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct MetersPerSecond(pub FloatingPoint);
#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    #[rstest]
    #[case(0.0, Meters(0.0))]
    #[case(1.0, Meters(1.0))]
    fn convert_meters(#[case] input: FloatingPoint, #[case] output: Meters) {
        assert_eq!(Meters::from(input), output);
    }
    #[rstest]
    #[case(1.0, Hertz(1.0))]
    #[case(2.0, Hertz(2.0))]
    fn convert_hertz(#[case] input: FloatingPoint, #[case] output: Hertz) {
        assert_eq!(Hertz::from(input), output);
    }
    #[rstest]
    #[case(1.0, Latitude(1.0))]
    #[case(2.0, Latitude(2.0))]
    fn convert_latitude(#[case] input: FloatingPoint, #[case] output: Latitude) {
        assert_eq!(Latitude::from(input), output);
    }
    #[rstest]
    #[case(1.0, Longitude(1.0))]
    #[case(2.0, Longitude(2.0))]
    fn convert_longitude(#[case] input: FloatingPoint, #[case] output: Longitude) {
        assert_eq!(Longitude::from(input), output);
    }
}
