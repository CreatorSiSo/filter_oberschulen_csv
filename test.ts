const districts = await (
	await fetch("https://schuldatenbank.sachsen.de/api/v1/key_tables/districts")
).json();

console.log(districts);

export {};
