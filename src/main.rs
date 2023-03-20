use dialoguer::console;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, MultiSelect};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct District {
	key: String,
	name: String,
}

fn get_districts() -> anyhow::Result<Vec<District>> {
	let res = ureq::get("https://schuldatenbank.sachsen.de/api/v1/key_tables/districts?fields[0]=key&fields[1]=name").call()?;
	let json = res.into_string()?;
	let districts = serde_json::from_str::<Vec<District>>(&json)?
		.into_iter()
		.filter(|district| district.key.len() == 5 && district.key != "00000")
		.collect();
	Ok(districts)
}

#[derive(Debug, Serialize, Deserialize)]
struct SchoolType {
	key: String,
	label: String,
}

fn get_school_types() -> anyhow::Result<Vec<SchoolType>> {
	let res =
		ureq::get("https://schuldatenbank.sachsen.de/api/v1/key_tables/school_types?limit=99&fields[0]=key&fields[1]=label").call()?;
	let json = res.into_string()?;
	Ok(serde_json::from_str(&json)?)
}

#[derive(Debug, Serialize, Deserialize)]
struct School {
	institution_key: u32,
	name: String,
	abbreviation: String,
	school_portal_mail: String,
	school_type_keys: String,
	building_name: String,
	building_id: u32,
	street: String,
	street_name: String,
	house_number: String,
	postcode: String,
	phone_code_1: Option<String>,
	phone_number_1: Option<String>,
	mail: Option<String>,
	homepage: Option<String>,
}

fn get_schools_of_type_in_district(
	school_types: &[&SchoolType],
	district: &District,
) -> anyhow::Result<Vec<School>> {
	let url = format!(
		"https://schuldatenbank.sachsen.de/api/v1/schools?limit=999&district_key={}&format=csv",
		district.key
	);
	let res = ureq::get(&url).call()?;
	let string_reader = res.into_reader();
	let mut csv_reader = csv::Reader::from_reader(string_reader);

	let mut schools = vec![];
	for maybe_school in csv_reader.deserialize() {
		let school: School = maybe_school?;
		let mut expected_school_types = school.school_type_keys.split(", ");
		let correct_school_type = school_types
			.iter()
			.any(|school_type| expected_school_types.any(|key| school_type.key == *key));
		if correct_school_type {
			schools.push(school);
		}
	}

	Ok(schools)
}

fn main() -> anyhow::Result<()> {
	let school_types = get_school_types()?;
	let school_type_labels: Vec<&str> = school_types
		.iter()
		.map(|school_type| school_type.label.as_str())
		.collect();

	let mut selected_school_types: Vec<&SchoolType>;
	loop {
		let school_type_selection = MultiSelect::with_theme(&ColorfulTheme::default())
			.with_prompt("Pick a School Type")
			.items(&school_type_labels)
			.max_length(5)
			.interact()?;

		selected_school_types = school_types
			.iter()
			.enumerate()
			.filter_map(|(i, school_type)| {
				school_type_selection.contains(&i).then_some(school_type)
			})
			.collect();

		if selected_school_types.is_empty() {
			println!(
				"  {}",
				console::style("At least one school type must be selected!")
					.bold()
					.red()
					.bright()
			);
			continue;
		}
		break;
	}

	let districts = get_districts()?;
	let district_names: Vec<&str> = districts
		.iter()
		.map(|district| district.name.as_str())
		.collect();
	let district_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
		.with_prompt("Pick a District")
		.default(0)
		.items(&district_names)
		.max_length(5)
		.interact()?;
	let selected_district = &districts[district_selection];

	let mut schools = get_schools_of_type_in_district(&selected_school_types, selected_district)?;
	println!(
		"  Downloaded data of {} schools",
		console::style(schools.len().to_string()).cyan()
	);
	schools.sort_by(|school_l, school_r| school_l.institution_key.cmp(&school_r.institution_key));

	let out_location: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
		.with_prompt("Output file name")
		.with_initial_text("./out.csv")
		.interact()?;
	let out_path = Path::new(&out_location);

	let mut writer = csv::Writer::from_path(out_path)?;
	for school in schools {
		writer.serialize(school)?;
	}
	writer.flush()?;

	println!("  Wrote data to disk",);

	Ok(())
}
