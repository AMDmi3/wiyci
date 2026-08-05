// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use time::OffsetDateTime;

pub trait FormatElapsed {
    fn format_elapsed(&self) -> String;
}

impl FormatElapsed for OffsetDateTime {
    fn format_elapsed(&self) -> String {
        let secs = (OffsetDateTime::now_utc() - *self).whole_seconds();

        let (count, unit) = match secs {
            ..60 => return "just now".to_string(),
            60..3600 => (secs / 60, "min"),
            3600..86400 => (secs / 3600, "hr"),
            86400.. => (secs / 86400, "day"),
        };

        if count == 1 {
            format!("{count} {unit}")
        } else {
            format!("{count} {unit}s")
        }
    }
}
