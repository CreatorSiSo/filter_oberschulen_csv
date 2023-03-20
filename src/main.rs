use serde::{Deserialize, Serialize};
use std::{env::args, io, path::Path};

#[derive(Debug, Serialize, Deserialize)]
struct District {
	key: String,
	name: String,
	#[serde(rename = "type")]
	type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SchoolType {
	key: String,
	abbreviation: String,
	label: String,
	school_category_key: String,
}

fn get_districts() -> anyhow::Result<Vec<District>> {
	let res = ureq::get("https://schuldatenbank.sachsen.de/api/v1/key_tables/districts").call()?;
	let json = res.into_string()?;
	let districts = serde_json::from_str::<Vec<District>>(&json)?
		.into_iter()
		.filter(|district| district.key.len() == 5 && district.key != "00000")
		.collect();
	Ok(districts)
}

fn get_school_types() -> anyhow::Result<Vec<SchoolType>> {
	let res =
		ureq::get("https://schuldatenbank.sachsen.de/api/v1/key_tables/school_types").call()?;
	let json = res.into_string()?;
	Ok(serde_json::from_str(&json)?)
}

fn main() -> anyhow::Result<()> {
	let mut args = args().skip(1);

	let out_str = args
		.next()
		.expect("Expected path to output file as first argument");
	let out_path = Path::new(&out_str);

	let districts = get_districts()?;
	for district in districts {
		println!("{}: {}", district.key, district.name);
	}
	let school_types = get_school_types()?;
	for school_type in school_types {
		println!("{}: {}", school_type.key, school_type.label);
	}

	// let mut writer = csv::Writer::from_path(out_path)?;

	// writer.flush()?;
	Ok(())
}
