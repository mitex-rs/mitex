use super::{ArchivedCommandSpecRepr, CommandSpecRepr};
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::{AlignedVec, Deserialize};

enum RkyvStreamData<'a> {
    Aligned(&'a [u8]),
    Unaligned(AlignedVec),
}

impl<'a> AsRef<[u8]> for RkyvStreamData<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Aligned(v) => v,
            Self::Unaligned(v) => v.as_slice(),
        }
    }
}

pub struct BytesModuleStream<'a> {
    data: RkyvStreamData<'a>,
}

impl<'a> BytesModuleStream<'a> {
    pub fn from_slice(v: &'a [u8]) -> Self {
        let v = if (v.as_ptr() as usize) % AlignedVec::ALIGNMENT != 0 {
            let mut aligned = AlignedVec::with_capacity(v.len());
            aligned.extend_from_slice(v);
            RkyvStreamData::Unaligned(aligned)
        } else {
            RkyvStreamData::Aligned(v)
        };

        Self { data: v }
    }

    #[cfg(feature = "rkyv-validation")]
    pub fn checkout(&self) -> &ArchivedCommandSpecRepr {
        rkyv::check_archived_root::<CommandSpecRepr>(self.data.as_ref()).unwrap()
    }

    /// # Safety
    /// The data source must be trusted and valid.
    pub unsafe fn checkout_unchecked(&self) -> &ArchivedCommandSpecRepr {
        rkyv::archived_root::<CommandSpecRepr>(self.data.as_ref())
    }

    #[cfg(feature = "rkyv-validation")]
    pub fn checkout_owned(&self) -> CommandSpecRepr {
        let v = self.checkout();
        let mut dmap = SharedDeserializeMap::default();
        v.deserialize(&mut dmap).unwrap()
    }

    /// # Safety
    /// The data source must be trusted and valid.
    pub unsafe fn checkout_owned_unchecked(&self) -> CommandSpecRepr {
        let v = self.checkout_unchecked();
        let mut dmap = SharedDeserializeMap::default();
        v.deserialize(&mut dmap).unwrap()
    }
}
