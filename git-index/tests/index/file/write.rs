use crate::index::file::read::loose_file_path;
use filetime::FileTime;
use git_index::verify::extensions::no_find;
use git_index::{decode, extension, write, State, Version};
use std::cmp::{max, min};

#[test]
fn roundtrips() -> crate::Result {
    enum Kind {
        Generated(&'static str),
        Loose(&'static str),
    }
    use Kind::*;
    let input = [
        (Loose("very-long-path"), write::Options::default(), false), // unclear why the file is smaller when written back
        (Generated("v2"), write::Options::default(), true),
        (Generated("V2_empty"), write::Options::default(), true),
        (
            Generated("v2_more_files"),
            write::Options {
                extensions: write::Extensions::Given {
                    end_of_index_entry: false,
                    tree_cache: true,
                },
                ..write::Options::default()
            },
            true,
        ),
    ];

    for (fixture, options, compare_byte_by_byte) in input {
        let (path, fixture) = match fixture {
            Generated(name) => (crate::fixture_index_path(name), name),
            Loose(name) => (loose_file_path(name), name),
        };
        let expected = git_index::File::at(&path, decode::Options::default())?;
        let expected_bytes = std::fs::read(&path)?;
        let mut out_bytes = Vec::new();

        expected.write_to(&mut out_bytes, options)?;
        let (actual, _) = State::from_bytes(&out_bytes, FileTime::now(), decode::Options::default())?;

        compare_states(&actual, &expected, options, fixture);
        if compare_byte_by_byte {
            compare_raw_bytes(&out_bytes, &expected_bytes, fixture);
        }
    }
    Ok(())
}

#[test]
fn v2_index_no_extensions() {
    let input = [
        "V2_empty",
        "v2",
        "v2_more_files",
        "v2_split_index",
        "v4_more_files_IEOT",
    ];

    for fixture in input {
        let path = crate::fixture_index_path(fixture);
        let expected = git_index::File::at(&path, decode::Options::default()).unwrap();

        let mut out = Vec::<u8>::new();
        let options = write::Options {
            hash_kind: git_hash::Kind::Sha1,
            version: Version::V2,
            extensions: write::Extensions::None,
        };

        expected.write_to(&mut out, options).unwrap();

        let (generated, _) = State::from_bytes(&out, FileTime::now(), decode::Options::default()).unwrap();
        compare_states(&generated, &expected, options, fixture);
    }
}

#[test]
fn v2_index_tree_extensions() {
    let input = [
        "V2_empty",
        "v2",
        "v2_more_files",
        "v2_split_index",
        "v4_more_files_IEOT",
    ];

    for fixture in input {
        let path = crate::fixture_index_path(fixture);
        let expected = git_index::File::at(&path, decode::Options::default()).unwrap();

        let mut out = Vec::<u8>::new();
        let options = write::Options {
            hash_kind: git_hash::Kind::Sha1,
            version: Version::V2,
            extensions: write::Extensions::Given {
                tree_cache: true,
                end_of_index_entry: false,
            },
        };

        expected.write_to(&mut out, options).unwrap();

        let (generated, _) = State::from_bytes(&out, FileTime::now(), decode::Options::default()).unwrap();
        compare_states(&generated, &expected, options, fixture);
    }
}

#[test]
fn v2_index_eoie_extensions() {
    let input = [
        "V2_empty",
        "v2",
        "v2_more_files",
        "v2_split_index",
        "v4_more_files_IEOT",
    ];

    for fixture in input {
        let path = crate::fixture_index_path(fixture);
        let expected = git_index::File::at(&path, decode::Options::default()).unwrap();

        let mut out = Vec::<u8>::new();
        let options = write::Options {
            hash_kind: git_hash::Kind::Sha1,
            version: Version::V2,
            extensions: write::Extensions::Given {
                tree_cache: false,
                end_of_index_entry: true,
            },
        };

        expected.write_to(&mut out, options).unwrap();

        let (generated, _) = State::from_bytes(&out, FileTime::now(), decode::Options::default()).unwrap();
        compare_states(&generated, &expected, options, fixture);
    }
}

fn compare_states(actual: &State, expected: &State, options: write::Options, fixture: &str) {
    actual.verify_entries().expect("valid");
    actual.verify_extensions(false, no_find).expect("valid");

    assert_eq!(actual.version(), options.version, "version mismatch in {}", fixture);
    assert_eq!(
        actual.tree(),
        options
            .extensions
            .should_write(extension::tree::SIGNATURE)
            .and_then(|_| expected.tree()),
        "tree extension mismatch in {}",
        fixture
    );
    assert_eq!(
        actual.entries().len(),
        expected.entries().len(),
        "entry count mismatch in {}",
        fixture
    );
    assert_eq!(actual.entries(), expected.entries(), "entries mismatch in {}", fixture);
    assert_eq!(
        actual.path_backing(),
        expected.path_backing(),
        "path_backing mismatch in {}",
        fixture
    );
}

fn compare_raw_bytes(generated: &[u8], expected: &[u8], fixture: &str) {
    assert_eq!(generated.len(), expected.len(), "file length mismatch in {}", fixture);

    let print_range = 10;
    for (index, (a, b)) in generated.iter().zip(expected.iter()).enumerate() {
        if a != b {
            let range_left = max(index - print_range, 0);
            let range_right = min(index + print_range, generated.len());
            let generated = &generated[range_left..range_right];
            let expected = &expected[range_left..range_right];

            panic! {"\n\nRoundtrip failed for index in fixture {:?} at position {:?}\n\
            \t   Input: ... {:?} ...\n\
            \tExpected: ... {:?} ...\n\n\
            ", &fixture, index, generated, expected}
        }
    }
}
