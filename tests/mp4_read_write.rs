mod util;

use lofty::{FileType, ItemKey, ItemValue, Probe, TagItem, TagType};

#[test]
fn mp4_read() {
	// This file contains an ilst atom
	let file = Probe::new().read_from_path("tests/assets/a.m4a").unwrap();

	assert_eq!(file.file_type(), &FileType::MP4);

	// Verify the ilst tag
	crate::verify_artist!(file, primary_tag, "Foo artist", 1);
}

#[test]
fn mp4_write() {
	let mut file = std::fs::OpenOptions::new()
		.read(true)
		.write(true)
		.open("tests/assets/a.m4a")
		.unwrap();

	let mut tagged_file = Probe::new().read_from(&mut file).unwrap();

	assert_eq!(tagged_file.file_type(), &FileType::MP4);

	// ilst
	crate::set_artist!(tagged_file, tag_mut, TagType::Mp4Atom, "Foo artist", 1 => file, "Bar artist");

	// Now reread the file
	let mut tagged_file = Probe::new().read_from(&mut file).unwrap();

	crate::set_artist!(tagged_file, tag_mut, TagType::Mp4Atom, "Bar artist", 1 => file, "Foo artist");
}
