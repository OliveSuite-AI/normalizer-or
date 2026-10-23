//! Shared replayable property-test configuration; excluded from production builds.
use proptest::test_runner::{Config, RngSeed};

/// Honor CI/local overrides; default to the deterministic PR profile.
pub fn config() -> Config {
    let mut config = Config {
        failure_persistence: Some(Box::new(
            proptest::test_runner::FileFailurePersistence::WithSource("proptest-regressions"),
        )),
        ..Config::default()
    };
    if std::env::var_os("PROPTEST_CASES").is_none() {
        config.cases = 512;
    }
    if std::env::var_os("PROPTEST_RNG_SEED").is_none() {
        config.rng_seed = RngSeed::Fixed(1729);
    }
    config
}
