// if you're handling more than terabytes of data, it's the future and you ought to update this code!
use chrono::{DateTime, Utc, NaiveDateTime};
use eframe::egui::Color32;

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

pub fn unpack_color(c: u32) -> Color32 {
	let r : u8 = (c >> 0) as u8;
	let g : u8 = (c >> 8) as u8;
	let b : u8 = (c >> 16) as u8;
	let a : u8 = (c >> 24) as u8;
	return Color32::from_rgba_unmultiplied(r, g, b, a);
}

pub fn repack_color(c: Color32) -> u32 {
	let mut out : u32 = 0;
	let mut offset = 0;
	for el in c.to_array() {
		out |= ((el & 0xFF) as u32) << offset;
		offset += 8;
	}
	return out;
}