use std::convert::TryInto;

#[derive(
    Debug,
    strum_macros::IntoStaticStr,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
    smart_default::SmartDefault,
)]
#[strum(serialize_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Plaintext,
    Json,
}

#[derive(
    Debug,
    strum_macros::IntoStaticStr,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
    smart_default::SmartDefault,
)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionFormat {
    #[default]
    Base64,
    Hex,
}

#[derive(Clone, derive_more::AsRef)]
pub struct BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]> + Clone,
{
    inner: T,
}

impl<T> std::fmt::Debug for BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]> + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        near_primitives::serialize::to_base(self.inner.as_ref()).fmt(f)
    }
}

impl<T> std::fmt::Display for BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]> + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        near_primitives::serialize::to_base(self.inner.as_ref()).fmt(f)
    }
}

impl<T> std::str::FromStr for BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]> + Clone,
{
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::serialize::from_base(value)
                .map_err(|_| format!("The value `{}` is not a valid base58 sequence", value))?
                .as_slice()
                .try_into()
                .map_err(|_| {
                    format!(
                        "The value could not be parsed into {} object",
                        std::any::type_name::<T>()
                    )
                })?,
        })
    }
}

impl<T> BlobAsBase58String<T>
where
    for<'a> T: std::convert::TryFrom<&'a [u8]> + AsRef<[u8]> + Clone,
{
    pub fn into_inner(self) -> T {
        self.inner
    }
}
