use chrono::{DateTime, Datelike, Local, Timelike, Utc};

/// pretty formats ´date_time` as how long `date_time` is ago
/// eventhough `date_time` is in UTC, time diff is displayed from the time diff from machine timezone now to `date_time` in UTC
pub fn pretty_format_date_time(date_time: &DateTime<Utc>) -> String {
	let date_time_naive = date_time.naive_local();
	let now = Local::now().with_timezone(&Utc).naive_local();
	let diff = now.signed_duration_since(date_time_naive);

	let diff_mins = diff.num_minutes();
	if diff_mins < 60 {
		format!("{} mins ago", diff_mins)
	} else {
		let diff_days = diff.num_days();
		match diff_days {
			0 => format!(
				"{:02}:{:02} today",
				date_time_naive.hour(),
				date_time_naive.minute()
			),
			1 => format!(
				"{:02}:{:02} yesterday",
				date_time_naive.hour(),
				date_time_naive.minute()
			),
			_ => {
				let date_time_naive_year = date_time_naive.year();
				// TODO: support / register if to use DD.MM.YYYY or MM.DD.YYYY
				if now.year() == date_time_naive_year {
					format!(
						"{}.{}., {:02}:{:02}",
						date_time_naive.day(),
						date_time_naive.month(),
						date_time_naive.hour(),
						date_time_naive.minute()
					)
				} else if date_time_naive_year >= 2000 {
					format!(
						"{}.{}.{}, {:02}:{:02}",
						date_time_naive.day(),
						date_time_naive.month(),
						date_time_naive_year - 2000,
						date_time_naive.hour(),
						date_time_naive.minute()
					)
				} else {
					format!(
						"{}.{}.{}, {:02}:{:02}",
						date_time_naive.day(),
						date_time_naive.month(),
						date_time_naive_year,
						date_time_naive.hour(),
						date_time_naive.minute()
					)
				}
			}
		}
	}
}
