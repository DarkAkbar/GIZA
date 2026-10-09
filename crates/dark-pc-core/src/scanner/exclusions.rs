use std::path::{Component, Path};

pub fn path_matches_exclusion(path: &Path, exclusions: &[String]) -> bool {
    for component in path.components() {
        if let Component::Normal(name) = component {
            let segment = name.to_string_lossy();
            for exclusion in exclusions {
                if segment.eq_ignore_ascii_case(exclusion) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_recycle_bin_segment() {
        let path = Path::new("C:\\$Recycle.Bin\\foo");
        assert!(path_matches_exclusion(
            path,
            &["$Recycle.Bin".to_string()]
        ));
    }
}
