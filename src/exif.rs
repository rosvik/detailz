use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct ExifField {
    pub tag: String,
    pub value: String,
}

pub fn read_exif(path: &Path) -> Option<Vec<ExifField>> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    let fields: Vec<_> = exif
        .fields()
        .map(|f| ExifField {
            tag: f.tag.to_string(),
            value: f.display_value().with_unit(&exif).to_string(),
        })
        .collect();
    (!fields.is_empty()).then_some(fields)
}
