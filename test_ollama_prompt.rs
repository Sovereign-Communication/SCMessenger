pub fn on_dial_failure(&mut self) {
    self.attempt_count += 1;
    self.last_attempt_ts = Instant::now();
    let doubled = self.backoff_duration.as_secs() * 2;
    self.backoff_duration = Duration::from_secs(doubled.min(30));
    if self.attempt_count >= 3 { self.is_dead = true; }
}