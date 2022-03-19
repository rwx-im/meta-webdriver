use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not set process capabilities")]
    CapSetError(#[source] caps::errors::CapsError),
    #[error("Could not launch chromedriver")]
    ChromeDriverStartError(#[source] io::Error),
    #[error("WebDriver error")]
    WebDriverError(#[from] thirtyfour::error::WebDriverError),
    #[error("Missing parameter: {0}")]
    MissingParameter(&'static str),
}
