use std::{ffi::OsStr, io, path::Path};

fn main() -> io::Result<()> {
	let path = Path::new("./data");

	let mut csv_readers: Vec<_> = path
		.read_dir()?
		.flat_map(|maybe_entry| maybe_entry.map(|entry| entry.path()))
		.filter(|path| path.extension() == Some(OsStr::new("csv")))
		.map(|csv_path| csv::Reader::from_path(csv_path).unwrap())
		.collect();

	let mut writer = csv::Writer::from_path(Path::new("./combined.csv"))?;
	writer.write_record(csv_readers[0].headers()?)?;

	for record in csv_readers.iter_mut().flat_map(|reader| reader.records()) {
		writer.write_record(&record?)?;
	}

	writer.flush()?;
	Ok(())
}
