// if you're handling more than terabytes of data, it's the future and you ought to update this code!
use chrono::{DateTime, Utc, NaiveDateTime};

const PREFIXES: &'static [&'static str] = &["", "k", "M", "G", "T"]; 

pub fn human_size(size:u64) -> String {
	let mut buf : f64 = size as f64;
	let mut prefix : usize = 0;
	while buf > 1024.0 && prefix < PREFIXES.len() -1 {
		buf /= 1024.0;
		prefix += 1;
	}
	
	return format!("{:.3} {}B", buf, PREFIXES[prefix]);
}

pub fn timestamp_to_str(t:i64) -> String {
	format!(
		"{}",
		DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(t, 0), Utc)
			.format("%Y/%m/%d %H:%M:%S")
	)
}