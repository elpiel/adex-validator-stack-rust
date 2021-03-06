#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use std::error;
use std::fmt;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use domain::{IOError, RepositoryError};

pub struct MemoryRepository<S: Clone, V> {
    records: Arc<RwLock<Vec<S>>>,
    cmp: Arc<dyn Fn(&S, &V) -> bool + Send + Sync>,
}

impl<S: Clone, V> MemoryRepository<S, V> {
    /// The `cmp` function is used for `find`, `add` and `has`, where there
    /// is need to see if the record already exist based on a value. For example:
    /// If you have an Id field, this will be your value, and since you need to check
    /// if the new_record.id == record_in_storage_id, then you can do that with
    /// `|&record, id| record.id == id`. If you don't have this kind of constraints, like
    /// when you don't have ids or you don't want to check for repeating records (i.e. you
    /// have an even store of some sort) you can use something like:
    /// `|&record, should_match| should_match` and in `add` for example, always invoke it with
    /// `false`
    pub fn new(initial_records: &[S], cmp: Arc<dyn Fn(&S, &V) -> bool + Send + Sync>) -> Self {
        Self {
            records: Arc::new(RwLock::new(initial_records.to_vec())),
            cmp,
        }
    }

    pub fn list<F>(&self, limit: u32, page: u64, filter: F) -> Result<Vec<S>, MemoryRepositoryError>
    where
        F: Fn(&S) -> Option<S>,
    {
        // 1st page, start from 0
        let skip_results = ((page - 1) * u64::from(limit)) as usize;
        // take `limit` results
        let take = limit as usize;

        self.records
            .read()
            .map(|reader| {
                reader
                    .iter()
                    .filter_map(|record| filter(record))
                    .skip(skip_results)
                    .take(take)
                    .collect()
            })
            .map_err(MemoryRepositoryError::from)
    }

    pub fn list_all<F>(&self, filter: F) -> Result<Vec<S>, MemoryRepositoryError>
    where
        F: Fn(&S) -> Option<S>,
    {
        self.records
            .read()
            .map(|reader| reader.iter().filter_map(|record| filter(record)).collect())
            .map_err(MemoryRepositoryError::from)
    }

    pub fn has(&self, cmp_value: &V) -> Result<bool, MemoryRepositoryError> {
        match self.records.read() {
            Ok(reader) => {
                let result = reader.iter().find(|current| (self.cmp)(current, cmp_value));
                Ok(result.is_some())
            }
            Err(error) => Err(MemoryRepositoryError::from(error)),
        }
    }

    pub fn find(&self, cmp_value: &V) -> Result<Option<S>, MemoryRepositoryError> {
        self.records
            .read()
            .map(|reader| {
                reader
                    .iter()
                    .find(|current| (self.cmp)(current, cmp_value))
                    .map(Clone::clone)
            })
            .map_err(MemoryRepositoryError::from)
    }

    pub fn add(&self, cmp_value: &V, record: S) -> Result<(), MemoryRepositoryError> {
        if self.has(cmp_value)? {
            Err(MemoryRepositoryError::AlreadyExists)
        } else {
            match self.records.write() {
                Ok(mut writer) => {
                    writer.push(record);

                    Ok(())
                }
                Err(error) => Err(MemoryRepositoryError::from(error)),
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MemoryRepositoryError {
    Reading,
    Writing,
    AlreadyExists,
}

impl error::Error for MemoryRepositoryError {}

impl IOError for MemoryRepositoryError {}

impl fmt::Display for MemoryRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_type = match *self {
            MemoryRepositoryError::Reading => "reading",
            MemoryRepositoryError::Writing => "writing",
            MemoryRepositoryError::AlreadyExists => "already exist",
        };

        write!(
            f,
            "Error occurred when trying to acquire lock for: {}",
            error_type
        )
    }
}

impl<T> From<PoisonError<RwLockReadGuard<'_, T>>> for MemoryRepositoryError {
    fn from(_: PoisonError<RwLockReadGuard<'_, T>>) -> Self {
        MemoryRepositoryError::Reading
    }
}

impl<T> From<PoisonError<RwLockWriteGuard<'_, T>>> for MemoryRepositoryError {
    fn from(_: PoisonError<RwLockWriteGuard<'_, T>>) -> Self {
        MemoryRepositoryError::Writing
    }
}

impl Into<RepositoryError> for MemoryRepositoryError {
    fn into(self) -> RepositoryError {
        match &self {
            MemoryRepositoryError::Reading | MemoryRepositoryError::Writing => {
                RepositoryError::IO(Box::new(self))
            }
            // @TODO: Implement AlreadyExist Error
            MemoryRepositoryError::AlreadyExists => RepositoryError::User,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq)]
    struct Dummy(u8);

    #[test]
    fn init_add_has_list_testing() {
        let dummy_one = Dummy(1);
        let cmp = Arc::new(|lhs: &Dummy, rhs: &Dummy| lhs == rhs);
        let repo = MemoryRepository::new(&[dummy_one], cmp);

        // get a list of all records should return 1
        assert_eq!(
            1,
            repo.list(10, 1, |x| Some(*x))
                .expect("No error should happen here")
                .len()
        );
        // and that it exist
        assert_eq!(true, repo.has(&dummy_one).expect("has shouldn't fail here"));

        let dummy_two = Dummy(2);

        // check if a non-existing record returns false
        assert_eq!(
            false,
            repo.has(&dummy_two).expect("has shouldn't fail here")
        );

        assert_eq!(
            Ok(()),
            repo.add(&dummy_two, dummy_two),
            "Adding new record should succeed"
        );

        assert_eq!(
            MemoryRepositoryError::AlreadyExists,
            repo.add(&dummy_two, dummy_two)
                .expect_err("Adding the same record again should fail")
        );
    }

    #[test]
    fn list_multiple_pages_filtering_and_list_all() {
        let dummy_filter = |x: &Dummy| Some(*x);
        let dummy_one = Dummy(1);
        let dummy_two = Dummy(2);
        let cmp = Arc::new(|lhs: &Dummy, rhs: &Dummy| lhs == rhs);
        let repo = MemoryRepository::new(&[dummy_one, dummy_two], cmp);

        // get a list with limit 10 should return 2 records
        assert_eq!(
            2,
            repo.list(10, 1, dummy_filter)
                .expect("No error should happen here")
                .len()
        );

        // get a list with limit 1 and page 1 should return Dummy 1
        let dummy_one_result = repo
            .list(1, 1, dummy_filter)
            .expect("No error should happen here");
        assert_eq!(1, dummy_one_result.len());
        assert_eq!(dummy_one, dummy_one_result[0]);

        // get a list with limit 1 and page 2 should return Dummy 2
        let dummy_two_result = repo
            .list(1, 2, dummy_filter)
            .expect("No error should happen here");
        assert_eq!(1, dummy_two_result.len());
        assert_eq!(dummy_two, dummy_two_result[0]);

        // get a list filtering out Dummy > 2
        let dummy_three = Dummy(3);
        repo.add(&dummy_three, dummy_three)
            .expect("The Dummy(3) should be added");

        assert_eq!(3, repo.list(10, 1, dummy_filter).unwrap().len());

        let filtered_result = repo
            .list(10, 1, |x| if x.0 > 2 { None } else { Some(*x) })
            .expect("No error should happen here");

        assert_eq!(vec![dummy_one, dummy_two], filtered_result);

        let list_all = repo
            .list_all(dummy_filter)
            .expect("No error should happen here");

        assert_eq!(3, list_all.len())
    }
}
