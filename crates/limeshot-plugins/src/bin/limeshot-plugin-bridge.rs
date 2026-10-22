//! A plugin bridge on its own (`limeshot-plugin-bridge --plugin-bridge <socket>`), for tests.
//! The LimeShot app runs the same code when started with `--plugin-bridge`.

fn main() {
    #[cfg(unix)]
    {
        let args: Vec<String> = std::env::args().skip(1).collect();
        if let Some(socket) = limeshot_plugins::bridge::socket_arg(&args) {
            std::process::exit(limeshot_plugins::bridge::child_main(
                &socket,
                &limeshot_plugins::load_module,
            ));
        }
    }
    std::process::exit(2);
}
