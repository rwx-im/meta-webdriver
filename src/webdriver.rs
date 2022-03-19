use std::ffi::OsStr;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

use caps::CapSet;
use thirtyfour::prelude::DesiredCapabilities;
use thirtyfour::WebDriver;
use tracing::debug;

use crate::Error;

/// Spawns a `ChromeDriver` process
#[derive(Debug)]
pub struct ChromeDriver {
    child: Child,
    port: u16,
}

impl ChromeDriver {
    /// Opens a new `ChromeDriver` by executing `program` and setting the given `port`.
    pub fn new<S: AsRef<OsStr>>(program: S, port: u16) -> Result<ChromeDriver, Error> {
        debug!("Starting new chromedriver process");

        let mut cmd = Command::new(program);
        let cmd = unsafe {
            cmd.pre_exec(|| {
                // Drop process capabilities
                debug!("Clearing effective process capabilities");

                caps::clear(None, CapSet::Effective)
                    .map_err(Error::CapSetError)
                    .unwrap();

                Ok(())
            })
        };

        let cmd = cmd.arg(format!("--port={}", port));
        let cmd = cmd.arg("--verbose");

        debug!(?cmd, "Starting background process");
        let child = cmd.spawn().map_err(Error::ChromeDriverStartError)?;

        Ok(ChromeDriver { child, port })
    }

    /// Opens a new `WebDriver` connection to the `ChromeDriver`.
    pub async fn webdriver(&self) -> Result<WebDriver, Error> {
        let mut caps = DesiredCapabilities::chrome();

        caps.add_chrome_arg("--no-sandbox")?;
        caps.set_headless()?;

        let driver = WebDriver::new(&format!("http://localhost:{}", self.port), &caps).await?;

        Ok(driver)
    }

    /// Returns true if the chromedriver process has exited, false otherwise.
    pub fn has_exited(&mut self) -> bool {
        !matches!(self.child.try_wait(), Ok(None))
    }

    /// Kills the chromedriver process if it's running.
    pub fn kill(&mut self) {
        self.child.kill().unwrap();
    }
}

impl Drop for ChromeDriver {
    fn drop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            self.child.kill().unwrap();
        }
    }
}
