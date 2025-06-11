#[cfg(all(test, feature = "tokio"))]
mod tests {
    use fusio::path::Path;
    use tempfile::TempDir;
    use tonbo::{executor::tokio::TokioExecutor, DbOption, Projection, Record, DB};

    #[derive(Record, Debug, PartialEq)]
    pub struct TestRecord {
        #[record(primary_key)]
        pub key: i32,
        pub value: String,
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_remove_visibility_in_same_transaction() {
        let temp_dir = TempDir::new().unwrap();
        let path = Path::from_filesystem_path(temp_dir.path()).unwrap();

        // Create database
        let db: DB<TestRecord, TokioExecutor> = DB::new(
            DbOption::new(path, &TestRecordSchema),
            TokioExecutor::current(),
            TestRecordSchema,
        )
        .await
        .unwrap();

        // Insert a record and commit
        {
            let mut txn = db.transaction().await;
            txn.insert(TestRecord {
                key: 1,
                value: "test".to_string(),
            });
            txn.commit().await.unwrap();
        }

        // In a new transaction, remove the record and check visibility
        {
            let mut txn = db.transaction().await;

            // Verify the record exists before removal
            assert!(txn.get(&1, Projection::All).await.unwrap().is_some());

            // Remove the record
            txn.remove(1);

            // The record should NOT be visible after removal in the same transaction
            let result_after = txn.get(&1, Projection::All).await.unwrap();

            // This assertion fails: result_after is Some when it should be None
            assert!(
                result_after.is_none(),
                "Record should not be visible after removal in the same transaction"
            );
        }
    }
}
