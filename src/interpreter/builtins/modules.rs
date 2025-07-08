use std::collections::HashMap;
use crate::repository;

pub fn create_core_loader() -> repository::Loader {
    repository::Loader::Intrinsic(
        HashMap::from([
            (vec!["core".to_string()], include_str!("../../../monoteny/core.monoteny")),
            (vec!["core".to_string(), "bool".to_string()], include_str!("../../../monoteny/core/bool.monoteny")),
            (vec!["core".to_string(), "debug".to_string()], include_str!("../../../monoteny/core/debug.monoteny")),
            (vec!["core".to_string(), "run".to_string()], include_str!("../../../monoteny/core/run.monoteny")),
            (vec!["core".to_string(), "strings".to_string()], include_str!("../../../monoteny/core/strings.monoteny")),
            (vec!["core".to_string(), "transpilation".to_string()], include_str!("../../../monoteny/core/transpilation.monoteny")),
        ]),
    )
}

pub fn create_common_loader() -> repository::Loader {
    repository::Loader::Intrinsic(
        HashMap::from([
            (vec!["common".to_string()], include_str!("../../../monoteny/common.monoteny")),
            (vec!["common".to_string(), "debug".to_string()], include_str!("../../../monoteny/common/debug.monoteny")),
            (vec!["common".to_string(), "math".to_string()], include_str!("../../../monoteny/common/math.monoteny")),
            (vec!["common".to_string(), "precedence".to_string()], include_str!("../../../monoteny/common/precedence.monoteny")),
        ]),
    )
}
