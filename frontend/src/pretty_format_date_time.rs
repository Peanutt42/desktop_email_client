use chrono::{DateTime, Datelike, Local, Timelike, Utc};

/// pretty formats ´date_time` as how long `date_time` is ago, falling back to pretty formatting the date if long enough ago
/// `date_time` is in UTC as it is directly stored in sqlite db, the displayed date and time are in local system timezone
pub fn pretty_format_date_time(date_time: &DateTime<Utc>) -> String {
	// time difference is computed using UTC timezone
	let now = Local::now().with_timezone(&Utc);
	let diff = now.signed_duration_since(date_time);

	// concrete time and date (hour, day, month, ...) are displayed using local system timezone
	let date_time_local = date_time.with_timezone(&Local).naive_local();

	let diff_mins = diff.num_minutes();
	if diff_mins < 60 {
		format!("{} mins ago", diff_mins)
	} else {
		let diff_days = diff.num_days();
		match diff_days {
			0 => format!(
				"{:02}:{:02} today",
				date_time_local.hour(),
				date_time_local.minute()
			),
			1 => format!(
				"{:02}:{:02} yesterday",
				date_time_local.hour(),
				date_time_local.minute()
			),
			_ => {
				let date_time_local_year = date_time_local.year();
				// TODO: support / register if to use DD.MM.YYYY or MM.DD.YYYY
				if now.year() == date_time_local_year {
					format!(
						"{}.{}., {:02}:{:02}",
						date_time_local.day(),
						date_time_local.month(),
						date_time_local.hour(),
						date_time_local.minute()
					)
				} else if date_time_local_year >= 2000 {
					format!(
						"{}.{}.{}, {:02}:{:02}",
						date_time_local.day(),
						date_time_local.month(),
						date_time_local_year - 2000,
						date_time_local.hour(),
						date_time_local.minute()
					)
				} else {
					format!(
						"{}.{}.{}, {:02}:{:02}",
						date_time_local.day(),
						date_time_local.month(),
						date_time_local_year,
						date_time_local.hour(),
						date_time_local.minute()
					)
				}
			}
		}
	}
}
