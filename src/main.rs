#![feature(iterator_try_collect)]

use dialoguer::console;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, MultiSelect};
use std::path::Path;

mod api;
use api::{Community, District, IndependetSchool, PublicSchool, School, SchoolType};

fn get_districts() -> anyhow::Result<Vec<District>> {
	let res = ureq::get("https://schuldatenbank.sachsen.de/api/v1/key_tables/districts?fields[0]=key&fields[1]=name").call()?;
	let json = res.into_string()?;
	let districts = serde_json::from_str::<Vec<District>>(&json)?
		.into_iter()
		.filter(|district| district.key.len() == 5 && district.key != "00000")
		.collect();
	Ok(districts)
}

fn get_communities() -> anyhow::Result<Vec<Community>> {
	let res =
		ureq::get("https://schuldatenbank.sachsen.de/api/v1/key_tables/communities?limit=999")
			.call()?;
	let json = res.into_string()?;
	let communities = serde_json::from_str::<Vec<Community>>(&json)?
		.into_iter()
		.filter(|community| community.key != "00000000")
		.collect();
	Ok(communities)
}

fn get_school_types() -> anyhow::Result<Vec<SchoolType>> {
	let res =
		ureq::get("https://schuldatenbank.sachsen.de/api/v1/key_tables/school_types?limit=99&fields[0]=key&fields[1]=label").call()?;
	let json = res.into_string()?;
	Ok(serde_json::from_str(&json)?)
}

fn get_public_schools_in_district(district: &District) -> anyhow::Result<Vec<PublicSchool>> {
	let url = format!(
		"https://schuldatenbank.sachsen.de/api/v1/schools?format=json&limit=999&district_key={}&only_schools=yes",
		district.key
	);
	let reader = ureq::get(&url).call()?.into_reader();
	let schools = serde_json::from_reader(reader)?;
	Ok(schools)
}

fn get_independent_schools() -> anyhow::Result<Vec<IndependetSchool>> {
	let url = "https://schuldatenbank.sachsen.de/api/v1/schools/independent?limit=999";
	let reader = ureq::get(&url).call()?.into_reader();
	let schools = serde_json::from_reader(reader)?;
	Ok(schools)
}

fn get_schools_of_types_in_district(
	school_types: &[&SchoolType],
	district: &District,
	all_communities: &[Community],
) -> anyhow::Result<Vec<School>> {
	let community_names_in_district: Vec<&str> = all_communities
		.iter()
		.filter(|community| community.key.starts_with(&district.key))
		.map(|community| community.name.as_str())
		.collect();

	let mut schools: Vec<School> = get_public_schools_in_district(district)?
		.into_iter()
		.map(|pub_school| pub_school.into_school(school_types))
		.collect();
	let mut indep_schools: Vec<School> = get_independent_schools()?
		.into_iter()
		.filter(|indep_school| {
			if schools
				.iter()
				.any(|school| school.institution_key == indep_school.institution_key)
			{
				return false;
			}
			for community_name in &community_names_in_district {
				if *community_name == indep_school.community {
					return true;
				}
			}
			false
		})
		.map(|indep_school| indep_school.into_school(school_types))
		.collect();
	schools.append(&mut indep_schools);

	let schools: Vec<School> = schools
		.into_iter()
		.filter(|school| {
			school
				.school_types
				.iter()
				.any(|school_type| school_types.contains(&school_type))
		})
		.collect();

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

	let communities = get_communities()?;
	let mut schools =
		get_schools_of_types_in_district(&selected_school_types, selected_district, &communities)?;
	println!(
		"  Downloaded data of {} public schools",
		console::style(schools.len().to_string()).cyan()
	);
	schools.sort_by(|school_l, school_r| school_l.institution_key.cmp(&school_r.institution_key));

	let out_location: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
		.with_prompt("Output file name")
		.with_initial_text("./out.csv")
		.interact()?;
	let mut writer = csv::Writer::from_path(Path::new(&out_location))?;

	for school in schools {
		writer.serialize(school)?;
	}

	writer.flush()?;
	println!("  Wrote data to disk",);

	Ok(())
}
