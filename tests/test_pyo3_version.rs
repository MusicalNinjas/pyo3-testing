//! Test to ensure CI fails if pyo3 version is bumped, so we can have a valid available version
//! for every pyo3 going forwards.

use std::{num::NonZeroU64, str::FromStr};

use cargo_metadata::{MetadataCommand, PackageId};
use semver::Version;

/// Grabbed from cargo (crate), this is sadly hidden in a private module
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub enum SemverCompatibility {
    Major(NonZeroU64),
    Minor(NonZeroU64),
    Patch(u64),
}

impl From<&Version> for SemverCompatibility {
    fn from(ver: &Version) -> Self {
        if let Some(m) = NonZeroU64::new(ver.major) {
            return SemverCompatibility::Major(m);
        }
        if let Some(m) = NonZeroU64::new(ver.minor) {
            return SemverCompatibility::Minor(m);
        }
        SemverCompatibility::Patch(ver.patch)
    }
}

/// Parse a PackageId to get the version. V ugly but not worth a newtype to impl Try_From
fn version(pkg: &PackageId) -> Version {
    let v = format!("{pkg}");
    let v = v.split_once("@").unwrap().1;
    Version::from_str(v).unwrap()
}

#[test]
fn version_numbers_match() {
    let cargo = MetadataCommand::new().exec().unwrap();
    let pyo3testing = cargo
        .packages
        .iter()
        .find(|pkg| pkg.name == "pyo3-testing")
        .unwrap()
        .version
        .clone();
    let deps = cargo.resolve.unwrap();
    let root = &deps[deps.root.as_ref().unwrap()];
    let pyo3 = &root.deps.iter().find(|pkg| pkg.name == "pyo3").unwrap().pkg;
    let pyo3 = version(pyo3);
    dbg!(&pyo3);
    dbg!(&pyo3testing);
    assert_eq!(
        SemverCompatibility::from(&pyo3),
        SemverCompatibility::from(&pyo3testing)
    );
}
