#[cfg(test)]
mod tests {
    use super::super::super::fetcher::{FetchInfo, Fetcher};
    use prelude::chrono::{TimeDelta, prelude::*};
    use pretty_assertions::assert_eq;
    #[test]
    fn simple_fetch() {
        let before = Utc::now();
        let network = Fetcher::new()
            .expect("should fetch")
            .fetch_network(
                &FetchInfo::default()
                    .network("AK".to_string())
                    .oldest_fetch(TimeDelta::zero()),
            )
            .expect("should fetch AK stations");
        assert_eq!(network.len(), 1);
        assert!(network[0].fetch_date >= before);
        assert_eq!(network[0].code, "AK");
    }
}
