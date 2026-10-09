//! MODULE 2 — File Scanner

mod engine;
mod exclusions;

pub use engine::{run_scan, ScanOptions, ScanProgress};
pub use exclusions::path_matches_exclusion;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::AtomicBool;

    use super::*;
    use crate::db::scan_repo;
    use crate::db::Database;

    #[test]
    fn scan_fixture_directory() {
        let temp = tempfile::tempdir().unwrap();
        let data_dir = temp.path().join("data");
        fs::create_dir(&data_dir).unwrap();
        fs::write(data_dir.join("a.txt"), b"hello").unwrap();
        fs::write(data_dir.join("b.txt"), b"world!").unwrap();

        let db_path = temp.path().join("scan.db");
        let mut db = Database::open(&db_path).unwrap();
        let scan_id =
            scan_repo::create_scan(db.conn_mut(), data_dir.to_str().unwrap()).unwrap();

        let cancel = AtomicBool::new(false);
        let mut last_files = 0u64;
        run_scan(
            &mut db,
            scan_id,
            ScanOptions::with_default_exclusions(data_dir),
            &cancel,
            |progress| last_files = progress.files_seen,
        )
        .unwrap();

        assert_eq!(last_files, 2);
        let status: String = db
            .conn()
            .query_row(
                "SELECT status FROM scans WHERE id = ?1",
                [scan_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "completed");
    }

    #[test]
    fn scan_respects_user_exclusion() {
        use crate::db::exclusions_repo;
        use crate::exclusions::normalize_user_exclusion_path;

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("scan_root");
        let skip_dir = root.join("private");
        std::fs::create_dir_all(&skip_dir).unwrap();
        std::fs::write(root.join("visible.txt"), b"a").unwrap();
        std::fs::write(skip_dir.join("hidden.txt"), b"b").unwrap();

        let mut db = Database::open(temp.path().join("scan.db")).unwrap();
        let conn = db.conn_mut();
        let skip_path = normalize_user_exclusion_path(&skip_dir.to_string_lossy()).unwrap();
        exclusions_repo::insert_user_exclusion(conn, &skip_path).unwrap();

        let scan_id = scan_repo::create_scan(conn, &root.to_string_lossy()).unwrap();
        let cancel = AtomicBool::new(false);
        let options = ScanOptions::from_connection(conn, root.clone()).unwrap();
        assert_eq!(options.exclusions.user_paths.len(), 1);

        run_scan(&mut db, scan_id, options, &cancel, |_| {}).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM files WHERE scan_id = ?1",
                [scan_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        let name: String = db
            .conn()
            .query_row(
                "SELECT name FROM files WHERE scan_id = ?1",
                [scan_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "visible.txt");
    }
}
