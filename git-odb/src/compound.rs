use crate::{loose, pack};

pub struct Db {
    pub loose: loose::Db,
    pub packs: Vec<pack::Bundle>,
}

mod object {
    use crate::loose;

    #[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone)]
    pub enum Object<'a> {
        Loose(loose::Object),
        Borrowed(crate::borrowed::Object<'a>),
    }
}
pub use object::Object;

mod locate {
    use crate::{compound, loose, pack};
    use git_object::borrowed;
    use quick_error::quick_error;

    quick_error! {
        #[derive(Debug)]
        pub enum Error {
            Loose(err: loose::db::locate::Error) {
                display("An error occurred while obtaining an object from the loose object store")
                source(err)
                from()
            }
            Pack(err: pack::bundle::locate::Error) {
                display("An error occurred while obtaining an object from the packed object store")
                source(err)
                from()
            }
        }
    }

    impl compound::Db {
        pub fn locate<'a>(
            &self,
            id: borrowed::Id<'_>,
            buffer: &'a mut Vec<u8>,
        ) -> Option<Result<compound::Object<'a>, Error>> {
            for pack in &self.packs {
                // See 8c5bd095539042d7db0e611460803cdbf172beb0 for a commit that adds polonius and makes the proper version compile.
                // See https://stackoverflow.com/questions/63906425/nll-limitation-how-to-work-around-cannot-borrow-buf-as-mutable-more-than?noredirect=1#comment113007288_63906425
                // The underlying issue is described here https://github.com/rust-lang/rust/issues/45402,
                // Once Polonius becomes a thing AND is not too slow, we must remove this double-lookup to become something like this:
                // if let Some(object) = if pack.locate(id, buffer, &mut pack::cache::DecodeEntryNoop) {…}
                if pack.locate(id, buffer, &mut pack::cache::DecodeEntryNoop).is_some() {
                    let object = pack.locate(id, buffer, &mut pack::cache::DecodeEntryNoop).unwrap();
                    return Some(object.map(compound::Object::Borrowed).map_err(Into::into));
                }
            }
            self.loose
                .locate(id)
                .map(|object| object.map(compound::Object::Loose).map_err(Into::into))
        }
    }
}

mod write {
    use crate::{compound, loose};
    use git_object::{owned, HashKind, Kind};
    use std::io::Read;

    impl crate::Write for compound::Db {
        type Error = loose::db::write::Error;

        fn write(&self, object: &owned::Object, hash: HashKind) -> Result<owned::Id, Self::Error> {
            self.loose.write(object, hash)
        }

        fn write_buf(&self, object: Kind, from: &[u8], hash: HashKind) -> Result<owned::Id, Self::Error> {
            self.loose.write_buf(object, from, hash)
        }

        fn write_stream(
            &self,
            kind: Kind,
            size: u64,
            from: impl Read,
            hash: HashKind,
        ) -> Result<owned::Id, Self::Error> {
            self.loose.write_stream(kind, size, from, hash)
        }
    }
}

mod init {
    use crate::{compound, loose, pack};
    use quick_error::quick_error;
    use std::path::PathBuf;

    quick_error! {
        #[derive(Debug)]
        pub enum Error {
            Pack(err: pack::bundle::Error) {
                display("Failed to instantiate a pack bundle")
                source(err)
                from()
            }
        }
    }

    /// Instantiation
    impl compound::Db {
        pub fn at(objects_directory: impl Into<PathBuf>) -> Result<compound::Db, Error> {
            let loose_objects = objects_directory.into();
            let packs = if let Ok(entries) = std::fs::read_dir(loose_objects.join("packs")) {
                let mut packs_and_sizes = entries
                    .filter_map(Result::ok)
                    .filter_map(|e| e.metadata().map(|md| (e.path(), md)).ok())
                    .filter(|(_, md)| md.file_type().is_file())
                    .filter(|(p, _)| p.extension().unwrap_or_default() == "idx" && p.starts_with("pack-"))
                    .map(|(p, md)| pack::Bundle::at(p).map(|b| (b, md.len())))
                    .collect::<Result<Vec<_>, _>>()?;
                packs_and_sizes.sort_by_key(|e| e.1);
                packs_and_sizes.into_iter().rev().map(|(b, _)| b).collect()
            } else {
                Vec::new()
            };

            Ok(compound::Db {
                loose: loose::Db::at(loose_objects),
                packs,
            })
        }
    }
}
