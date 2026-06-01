pub struct EventQuery {
    pub maximum_events_returned: Option<i64>,
    pub minimum_magnitude: Option<f32>,
    pub event_name: Option<String>,
}
impl EventQuery {
    pub fn with_maximum_events_returned(mut self, maximum_events_returned: i64) -> Self {
        assert!(maximum_events_returned.is_positive());
        self.maximum_events_returned = Some(maximum_events_returned);
        self
    }
    pub fn with_event_name(mut self, event_name: String) -> Self {
        self.event_name = Some(event_name);
        self
    }
}
impl Default for EventQuery {
    fn default() -> Self {
        Self {
            maximum_events_returned: None,
            minimum_magnitude: None,
            event_name: None,
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn event_query_empty_defaults() {
        let query = EventQuery::default();
        assert_eq!(query.maximum_events_returned, None);
        assert_eq!(query.event_name, None);
    }
    #[test]
    fn with_event_name() {
        for i in 0..3 {
            let name = i.to_string();
            let query = EventQuery::default().with_event_name(name.clone());
            assert_eq!(query.event_name, Some(name));
        }
    }
    #[test]
    fn event_query_update_defaults() {
        let m = 1000;
        let query = EventQuery::default().with_maximum_events_returned(m);
        assert_eq!(query.maximum_events_returned, Some(m));
    }
}
