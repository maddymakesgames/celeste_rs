use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use aliasable::boxed::AliasableBox;
use dotnetdll::{dll::DLLError, prelude::Resolution};

/// A self-referential type to hold the data used by [Resolution]
///
/// This is needed because [Resolution] only holds references into the raw dll binary.
/// Because we need the [FileProvider](super::FileProvider) api to be compatable with [ZipArchive](zip::ZipArchive) we need
/// exclusive access to the provider at all times. Thus we need all data stored in [Mod](super::Mod)
/// to be buffered.
///
/// So [Resolution] needs a reference into a buffer, but also the data comes from something
/// we need to be able to take mutable references to at all times, thus we need to buffer
/// the data ourselves and the only real way to do that is with a self-referential type.
///
/// The type is safe to interact with through any safe apis, since the only thing you can
/// access is the inner [Resolution] through the [Deref] and [DerefMut] impls.
pub struct BufferedDLL {
    inner: Resolution<'static>,
    _buf: AliasableBox<[u8]>,
}

impl BufferedDLL {
    pub fn new(bytes: impl Into<Box<[u8]>>) -> Result<BufferedDLL, DLLError> {
        let buf = AliasableBox::from_unique(bytes.into());
        let dll = Resolution::parse(&buf, dotnetdll::prelude::ReadOptions {
            skip_method_bodies: false,
        })?;

        // Extending to 'static is safe as long as we never touch
        // the buf.
        // since the field is not public and we only use BufferedDLL for the Resolution
        // this should be safe
        // Also because of field drop order the dll will be dropped before the box
        // So the pointer never dangles
        #[allow(clippy::missing_transmute_annotations)]
        let dll = unsafe { std::mem::transmute::<_, Resolution<'static>>(dll) };

        Ok(BufferedDLL {
            inner: dll,
            _buf: buf,
        })
    }
}

impl Deref for BufferedDLL {
    type Target = Resolution<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for BufferedDLL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Debug for BufferedDLL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
