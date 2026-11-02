pub fn end_minute(start_minute: u32, duration_minutes: u32) -> u32 {
    start_minute + duration_minutes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_minute_adds_duration_to_start() {
        let result = end_minute(540, 30);
        assert_eq!(result, 570);
    }
}
