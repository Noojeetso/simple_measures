use crate::measures::Algorithm;
use nix::errno::Errno;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult};
use std::process::exit;
use std::time::Duration;

pub unsafe fn call_long_running_function<'a, AlgArgT, ResT>(
    function: &Algorithm<'a, AlgArgT, ResT>,
    mut data: AlgArgT,
    threshold: Duration,
) -> bool {
    let mut result = false;

    let child_pid = match fork() {
        Ok(ForkResult::Child) => {
            match function {
                Algorithm::NonMutatingAlgorithm(function) => _ = function(&data),
                Algorithm::MutatingAlgorithm(function) => _ = function(&mut data),
            }
            exit(0);
        }

        Ok(ForkResult::Parent { child, .. }) => child,

        Err(err) => {
            panic!("[call_long_running_function] fork() failed: {}", err);
        }
    };

    let time = std::time::Instant::now();
    loop {
        std::thread::sleep(std::time::Duration::from_secs_f64(0.1));
        match waitpid(child_pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => {}

            Ok(_status) => {
                result = true;
                break;
            }

            Err(err) => panic!("[call_long_running_function] waitpid() failed: {}", err),
        }
        let took = time.elapsed();
        if took > threshold {
            match kill(child_pid, Signal::SIGKILL) {
                Ok(_) => {}
                Err(err) => {
                    if err != Errno::ESRCH {
                        panic!(
                            "[call_long_running_function] Error sending termination signal: {}",
                            err
                        );
                    }
                }
            }
            break;
        }
    }

    return result;
}
