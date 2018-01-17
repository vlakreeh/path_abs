use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::result;
use std::ffi::OsStr;
use stfu8;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::ffi::OsString;

use super::PathAbs;

impl Serialize for PathAbs {
    #[cfg(unix)]
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.as_os_str().as_bytes();
        let stfu = stfu8::encode_u8(&bytes);
        serializer.serialize_str(&stfu)
    }

    #[cfg(windows)]
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wide: Vec<u16> = self.as_os_str().encode_wide().iter().cloned().collect();
        let stfu = stfu8::encode_u16(&wide);
        serializer.serialize_str(&stfu)
    }
}

impl<'de> Deserialize<'de> for PathAbs {
    #[cfg(unix)]
    fn deserialize<D>(deserializer: D) -> result::Result<PathAbs, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p = String::deserialize(deserializer)?;
        let p = stfu8::decode_u8(&p)
            .map_err(serde::de::Error::custom)?;
        PathAbs::new(OsStr::from_bytes(&p)).map_err(serde::de::Error::custom)
    }

    #[cfg(windows)]
    fn deserialize<D>(deserializer: D) -> result::Result<PathAbs, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p = String::deserialize(deserializer)?;
        let p = stfu8::decode_u16(&p)
            .map_err(serde::de::Error::custom)?;
        PathAbs::new(OsString::from_wide(&p)).map_err(serde::de::Error::custom)
    }
}

#[test]
fn sanity_serde() {
    use serde_json;
    use tempdir::TempDir;

    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let tmp_abs = PathAbs::new(tmp_dir.path()).unwrap();

    let foo = PathAbs::create_file(tmp_abs.join("foo.txt")).unwrap();
    let bar_dir = PathAbs::create_dir(tmp_abs.join("bar")).unwrap();
    let foo_bar_dir = PathAbs::create_dir_all(tmp_abs.join("foo/bar")).unwrap();

    let expected = vec![
        foo,
        bar_dir,
        foo_bar_dir,
    ];

    let expected_str = format!("[\
         \"{0}/foo.txt\",\
         \"{0}/bar\",\
         \"{0}/foo/bar\"\
    ]", tmp_abs.to_string_lossy().as_ref());

    let result_str = serde_json::to_string(&expected).unwrap();
    assert_eq!(expected_str, result_str);

    let result: Vec<PathAbs> = serde_json::from_str(&result_str).unwrap();
    assert_eq!(expected, result);
}