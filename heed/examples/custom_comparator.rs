use std::cmp::Ordering;
use std::error::Error;
use std::path::Path;
use std::{fs, str};

use heed::EnvOpenOptions;
use heed_traits::Comparator;
use heed_types::{Str, Unit};

struct StringAsIntCmp;

// This function takes two strings which represent positive numbers,
// parses them into i32s and compare the parsed value.
// Therefore "-1000" < "-100" must be true even without '0' padding.
impl Comparator for StringAsIntCmp {
    fn compare(a: &[u8], b: &[u8]) -> Ordering {
        let a: i32 = str::from_utf8(a).unwrap().parse().unwrap();
        let b: i32 = str::from_utf8(b).unwrap().parse().unwrap();
        a.cmp(&b)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let env_path = Path::new("target").join("custom-key-cmp.mdb");

    let _ = fs::remove_dir_all(&env_path);

    fs::create_dir_all(&env_path)?;
    let env = EnvOpenOptions::new()
        .map_size(10 * 1024 * 1024) // 10MB
        .max_dbs(3)
        .open(env_path)?;

    let mut wtxn = env.write_txn()?;
    let db = env
        .database_options()
        .types::<Str, Unit>()
        .comparator::<StringAsIntCmp>()
        .create(&mut wtxn)?;
    wtxn.commit()?;

    let mut wtxn = env.write_txn()?;

    // We fill our database with entries.
    db.put(&mut wtxn, "-100000", &())?;
    db.put(&mut wtxn, "-10000", &())?;
    db.put(&mut wtxn, "-1000", &())?;
    db.put(&mut wtxn, "-100", &())?;
    db.put(&mut wtxn, "100", &())?;

    // We check that the key are in the right order ("-100" < "-1000" < "-10000"...)
    let mut iter = db.iter(&wtxn)?;
    assert_eq!(iter.next().transpose()?, Some(("-100000", ())));
    assert_eq!(iter.next().transpose()?, Some(("-10000", ())));
    assert_eq!(iter.next().transpose()?, Some(("-1000", ())));
    assert_eq!(iter.next().transpose()?, Some(("-100", ())));
    assert_eq!(iter.next().transpose()?, Some(("100", ())));
    drop(iter);

    Ok(())
}
