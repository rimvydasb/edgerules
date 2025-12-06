#[cfg(test)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod test {
    use log::info;
    use std::io::Write;
    use std::sync::Once;

    static INIT: Once = Once::new();

    // Intentionally minimal: only logging helpers are exposed here.

    pub fn init_test(name: &str) {
        init_logger();
        info!(">>> starting test {}", name);
    }

    pub fn init_logger() {
        INIT.call_once(|| {
            env_logger::builder()
                .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
                .init()
        })
    }
}
