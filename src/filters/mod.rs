use artushak_web_assets::assets::AssetFilterError;

pub mod scss2css;

#[derive(Debug)]
pub enum AssetFilterCustomError {
    InvalidInputCount(usize),
    RSASSError(Box<rsass::Error>),
}

impl AssetFilterError for AssetFilterCustomError {}

impl From<rsass::Error> for AssetFilterCustomError {
    fn from(err: rsass::Error) -> Self {
        Self::RSASSError(Box::new(err))
    }
}
