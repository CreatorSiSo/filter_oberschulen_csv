use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize)]
pub struct District {
	pub key: String,
	pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Community {
	pub key: String,
	pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolType {
	pub key: String,
	pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicSchool {
	pub institution_key: String,
	pub name: String,
	pub id: u32,
	pub abbreviation: String,
	pub institution_number: String,
	pub legal_status_key: String,
	pub inspectorate_key: String,
	pub company_number: Option<String>,
	pub school_category_key: String,
	pub headmaster_salutation_key: Option<char>,
	pub headmaster_firstname: Option<String>,
	pub headmaster_lastname: Option<String>,
	pub school_portal_mail: Option<String>,
	pub educational_concept_key: Option<u32>,
	pub school_property_key: Option<String>,
	pub opening_date: Option<String>,
	pub buildings: Vec<Building>,
}

impl PublicSchool {
	pub fn into_school(self, possible_school_types: &[&SchoolType]) -> School {
		let PublicSchool {
			institution_key,
			name,
			opening_date,
			buildings,
			..
		} = self;

		let school_types = buildings
			.iter()
			.map(|building| &building.school_type_keys)
			.flatten()
			.map(|key| {
				possible_school_types.iter().find_map(|school_type| {
					(school_type.key == key.to_string()).then_some((*school_type).clone())
				})
			})
			.flatten()
			.collect();

		let Building {
			street,
			street_name,
			house_number,
			postcode,
			community,
			mut longitude,
			mut latitude,
			mut phone_code,
			mut phone_number,
			mut phone_identifier,
			mut fax_code,
			mut fax_number,
			mut mail,
			mut homepage,
			..
		} = buildings[0].clone();
		for building in &buildings[1..] {
			if building.longitude.is_some() {
				longitude = building.longitude
			}
			if building.latitude.is_some() {
				latitude = building.latitude
			}
			if building.phone_code.is_some() {
				phone_code = building.phone_code.clone()
			}
			if building.phone_number.is_some() {
				phone_number = building.phone_number.clone()
			}
			if building.phone_identifier.is_some() {
				phone_identifier = building.phone_identifier.clone()
			}
			if building.fax_code.is_some() {
				fax_code = building.fax_code.clone()
			}
			if building.fax_number.is_some() {
				fax_number = building.fax_number.clone()
			}
			if building.mail.is_some() {
				mail = building.mail.clone()
			}
			if building.homepage.is_some() {
				homepage = building.homepage.clone()
			}
		}

		School {
			institution_key,
			name,
			school_types,
			opening_date,
			street,
			street_name,
			house_number,
			postcode,
			community,
			longitude,
			latitude,
			phone_code,
			phone_number,
			phone_identifier,
			fax_code,
			fax_number,
			mail,
			homepage,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndependetSchool {
	pub institution_key: String,
	pub name: String,
	#[serde(rename = "school_category")]
	pub school_category_name: String,
	#[serde(rename = "school_types")]
	pub school_type_name: String,
	pub owner_id: u32,
	pub educational_sector: Option<String>,
	pub inspectorate: String,
	pub opening_date: Option<String>,
	pub homepage: Option<String>,
	pub street: String,
	pub street_name: String,
	pub house_number: String,
	pub postcode: String,
	pub community: String,
}

impl IndependetSchool {
	pub fn into_school(self, possible_school_types: &[&SchoolType]) -> School {
		let IndependetSchool {
			institution_key,
			name,
			school_type_name,
			opening_date,
			homepage,
			street,
			street_name,
			house_number,
			postcode,
			community,
			..
		} = self;

		School {
			institution_key,
			name,
			school_types: possible_school_types
				.into_iter()
				.find_map(|school_type| {
					(school_type.label == school_type_name).then_some((*school_type).clone())
				})
				.into_iter()
				.collect(),
			opening_date,
			street,
			street_name,
			house_number,
			postcode,
			community,
			homepage,
			..Default::default()
		}
	}
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct School {
	pub institution_key: String,
	pub name: String,
	#[serde(serialize_with = "serialize_school_types")]
	#[serde(rename = "school_type_keys")]
	pub school_types: Vec<SchoolType>,
	pub opening_date: Option<String>,
	pub street: String,
	pub street_name: String,
	pub house_number: String,
	pub postcode: String,
	pub community: String,
	pub longitude: Option<f64>,
	pub latitude: Option<f64>,
	pub phone_code: Option<String>,
	pub phone_number: Option<String>,
	pub phone_identifier: Option<String>,
	pub fax_code: Option<String>,
	pub fax_number: Option<String>,
	pub mail: Option<String>,
	pub homepage: Option<String>,
}

fn serialize_school_types<S>(data: &[SchoolType], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(
		&data
			.into_iter()
			.map(|school_type| school_type.key.to_string())
			.collect::<Vec<String>>()
			.join(", "),
	)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
	pub building_name: Option<String>,
	pub street: String,
	pub street_name: String,
	pub house_number: String,
	pub postcode: String,
	pub community: String,
	pub longitude: Option<f64>,
	pub latitude: Option<f64>,
	#[serde(rename(deserialize = "phone_code_1"))]
	pub phone_code: Option<String>,
	#[serde(rename(deserialize = "phone_number_1"))]
	pub phone_number: Option<String>,
	#[serde(rename(deserialize = "phone_identifier_1"))]
	pub phone_identifier: Option<String>,
	pub fax_code: Option<String>,
	pub fax_number: Option<String>,
	pub school_type_keys: Vec<u32>,
	pub mail: Option<String>,
	pub homepage: Option<String>,
}
