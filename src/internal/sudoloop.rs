use std::thread;
use std::time::Duration;

use crate::ShellCommand;

/// Loop sudo so longer builds don't time out
#[allow(clippy::module_name_repetitions)]
pub fn start_sudoloop() {
    std::thread::spawn(|| loop {
        if prompt_sudo() {
            // Sudo prompt returned error
            break;
        }

        thread::sleep(Duration::from_secs(3 * 60));
    });
}

fn prompt_sudo() -> bool {
    ShellCommand::sudo().arg("-v").wait_success().is_err()
}
