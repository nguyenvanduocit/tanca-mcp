//! Minimal stderr logger using the Go `log` package's default format.

/// Prints `YYYY/MM/DD HH:MM:SS <message>` to stderr.
#[macro_export]
macro_rules! logf {
    ($($arg:tt)*) => {
        eprintln!(
            "{} {}",
            ::chrono::Local::now().format("%Y/%m/%d %H:%M:%S"),
            format_args!($($arg)*)
        )
    };
}

/// Formats a duration the way Go's `time.Duration.String` does for values
/// of one second or more (e.g. `1.25s`, `1m5.5s`), and in `ms` below that.
pub fn go_duration(d: std::time::Duration) -> String {
    let nanos = d.as_nanos();
    if nanos == 0 {
        return "0s".into();
    }
    if nanos < 1_000_000_000 {
        let ms = nanos as f64 / 1e6;
        return format!("{}ms", trim_float(ms));
    }
    let total_secs = nanos / 1_000_000_000;
    let frac = nanos % 1_000_000_000;
    let mut secs = format!("{}", total_secs % 60);
    if frac > 0 {
        let digits = format!("{frac:09}");
        secs.push('.');
        secs.push_str(digits.trim_end_matches('0'));
    }
    let mins = total_secs / 60;
    if mins == 0 {
        format!("{secs}s")
    } else if mins < 60 {
        format!("{mins}m{secs}s")
    } else {
        format!("{}h{}m{secs}s", mins / 60, mins % 60)
    }
}

fn trim_float(v: f64) -> String {
    let s = format!("{v:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::go_duration;
    use std::time::Duration;

    #[test]
    fn formats_like_go() {
        assert_eq!(go_duration(Duration::from_secs(1)), "1s");
        assert_eq!(go_duration(Duration::from_millis(1250)), "1.25s");
        assert_eq!(
            go_duration(Duration::from_nanos(2_000_000_001)),
            "2.000000001s"
        );
        assert_eq!(go_duration(Duration::from_millis(65_500)), "1m5.5s");
    }
}
