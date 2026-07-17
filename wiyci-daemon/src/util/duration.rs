// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Duration;

pub trait DurationExt {
    fn with_jitter(self, jitter: f64) -> Self;
    fn trimmed_to_micros(self) -> Self;
}

impl DurationExt for Duration {
    fn with_jitter(self, jitter: f64) -> Self {
        self.mul_f64(1.0 + jitter * rand::random::<f64>())
    }

    fn trimmed_to_micros(self) -> Self {
        Duration::from_micros(self.as_micros() as u64)
    }
}
