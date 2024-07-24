use crate::{declare_class_vtable, declare_interface};

use super::{
    macros::{declare_class_iid, def_class_iid},
    EInterface, Interface, FUID, TUID,
};
use bytemuck::{Pod, Zeroable};
use static_assertions::const_assert_eq;
use std::{
    fmt,
    marker::PhantomData,
    mem::{size_of, transmute, transmute_copy, MaybeUninit},
    ops::Deref,
    os::raw::c_void,
    ptr::NonNull,
    sync::atomic::AtomicU32,
};

pub struct ENoInterface;
pub struct EInterfacePtr;
macro_rules! declare_uid {
    ($name:tt, $l1:literal, $l2:literal, $l3:literal, $l4:literal) => {
        const $name: Tuid =
            $crate::plugininterfaces::base::funknown::inline_uid!($l1, $l2, $l3, $l4);
    };
}
macro_rules! inline_uid_of {
    ($class_name:tt) => {
        paste::paste! { [<$class_name _iid>]}
    };
}
macro_rules! inline_uid_from_fuid {
    ($fuid:ident) => {
        inline_uid!(
            $fuid.get_long_1(),
            $fuid.get_long_2(),
            $fuid.get_long_3(),
            $fuid.get_long_4()
        );
    };
}
// macro_rules! declare_funknown_methods {
//     () => {
//         #[allow(non_upper_case_globals)]
//         const _funknown_ref_count: std::sync::atomic::AtomicU32 =
//             std::sync::atomic::AtomicU32::new(0);
//     };
// }

pub(super) struct FUnknownPrivate {}
impl FUnknownPrivate {
    #[inline(always)]
    pub(super) fn iid_equal(iid1: *const c_void, iid2: *const c_void) -> bool {
        let p1 = iid1 as *const u64;
        let p2 = iid2 as *const u64;

        let (p1_0, p1_1, p2_0, p2_1) = unsafe {
            (
                *p1,
                *p1.byte_add(size_of::<u64>()),
                *p2,
                *p2.byte_add(size_of::<u64>()),
            )
        };

        p1_0 == p2_0 && p1_1 == p2_1
    }
}

/// Reference wrapper for FUID
#[repr(transparent)]
struct FUIDRef<'a>(&'a TUID);
// impl Deref for FUIDRef<'_> {
//     type Target = FUID;
//     fn deref(&self) -> &Self::Target {
//         unsafe { self.}
//     }
// }

// pub struct FUnknown {}

//def_class_iid!(FUnknown);

#[repr(transparent)]
pub struct FUnknown(NonNull<c_void>);
declare_class_iid!(FUnknown, 0x0000_0000, 0x0000_0000, 0xC000_0000, 0x0000_0046);
impl Clone for FUnknown {
    fn clone(&self) -> Self {
        unsafe { (self.vtable().add_ref)(transmute_copy(self)) };
        Self(self.0)
    }
}
impl Drop for FUnknown {
    fn drop(&mut self) {
        unsafe {
            (self.vtable().release)(transmute_copy(self));
        }
    }
}
impl PartialEq for FUnknown {
    fn eq(&self, other: &Self) -> bool {
        self.as_raw() == other.as_raw()
            || self.cast::<FUnknown>().unwrap().0 == other.cast::<FUnknown>().unwrap().0
    }
}
impl Eq for FUnknown {}
impl fmt::Debug for FUnknown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FUnknown").field(&self.as_raw()).finish()
    }
}
// declare_class_vtable!(FUnknown,
//     query_interface: unsafe fn (this: *mut c_void, iid: *const FUID, interface: *mut *mut c_void) -> Result<(), EInterface>,
//     add_ref: unsafe fn(this: *mut c_void) -> u32,
//     release: unsafe fn(this: *mut c_void) -> u32
// );
#[repr(C)]
pub struct FUnknownVtable {
    pub query_interface: unsafe fn(
        this: *mut c_void,
        iid: *const FUID,
        interface: *mut *mut c_void,
    ) -> Result<(), EInterface>,
    pub add_ref: unsafe fn(this: *mut c_void) -> u32,
    pub release: unsafe fn(this: *mut c_void) -> u32,
}
impl FUnknownVtable {
    pub const fn new<T: FUnknownImpl, const OFFSET: isize>() -> Self {
        unsafe fn query_interface<T: FUnknownImpl, const OFFSET: isize>(
            this: *mut c_void,
            iid: *const FUID,
            interface: *mut *mut c_void,
        ) -> Result<(), EInterface> {
            let this = (this as *mut *mut c_void).offset(OFFSET) as *mut T;
            (*this).query_interface(iid, interface)
        }
        unsafe fn add_ref<T: FUnknownImpl, const OFFSET: isize>(this: *mut c_void) -> u32 {
            let this = (this as *mut *mut c_void).offset(OFFSET) as *mut T;
            (*this).add_ref()
        }
        unsafe fn release<T: FUnknownImpl, const OFFSET: isize>(this: *mut c_void) -> u32 {
            let this = (this as *mut *mut c_void).offset(OFFSET) as *mut T;
            T::release(this)
        }
        Self {
            query_interface: query_interface::<T, OFFSET>,
            add_ref: add_ref::<T, OFFSET>,
            release: release::<T, OFFSET>,
        }
    }
}

unsafe impl Interface for FUnknown {
    type Vtable = FUnknownVtable;
    const iid: FUID = FUID { data: FUnknown_iid };
}

pub trait FUnknown_Type {
    fn query_interface(&self, iid: &TUID) -> Result<*const *const c_void, ENoInterface>;

    fn add_ref(&self) -> u32;

    fn release(&mut self) -> u32;
}

pub trait FUnknownImpl {
    type Impl;

    fn get_impl(&self) -> &Self::Impl;

    unsafe fn query_interface(
        &self,
        iid: *const FUID,
        interface: *mut *mut c_void,
    ) -> Result<(), EInterface>;

    fn add_ref(&self) -> u32;

    unsafe fn release(self_: *mut Self) -> u32;

    const INNER_OFFSET_IN_POINTERS: usize;
}

struct UID;
impl UID {
    pub const fn to_tuid<const P0: u32, const P1: u32, const P2: u32, const P3: u32>() -> TUID {
        let l1_1: u8 = ((P0 & 0xFF_00_00_00) >> 24) as u8;
        let l1_2: u8 = ((P0 & 0x00_FF_00_00) >> 16) as u8;
        let l1_3: u8 = ((P0 & 0x00_00_FF_00) >> 8) as u8;
        let l1_4: u8 = (P0 & 0x00_00_00_FF) as u8;
        let l2_1: u8 = ((P1 & 0xFF_00_00_00) >> 24) as u8;
        let l2_2: u8 = ((P1 & 0x00_FF_00_00) >> 16) as u8;
        let l2_3: u8 = ((P1 & 0x00_00_FF_00) >> 8) as u8;
        let l2_4: u8 = (P1 & 0x00_00_00_FF) as u8;
        let l3_1: u8 = ((P2 & 0xFF_00_00_00) >> 24) as u8;
        let l3_2: u8 = ((P2 & 0x00_FF_00_00) >> 16) as u8;
        let l3_3: u8 = ((P2 & 0x00_00_FF_00) >> 8) as u8;
        let l3_4: u8 = (P2 & 0x00_00_00_FF) as u8;
        let l4_1: u8 = ((P3 & 0xFF_00_00_00) >> 24) as u8;
        let l4_2: u8 = ((P3 & 0x00_FF_00_00) >> 16) as u8;
        let l4_3: u8 = ((P3 & 0x00_00_FF_00) >> 8) as u8;
        let l4_4: u8 = (P3 & 0x00_00_00_FF) as u8;

        [
            l1_4, l1_3, l1_2, l1_1, l2_2, l2_1, l2_4, l2_3, l3_1, l3_2, l3_3, l3_4, l4_1, l4_2,
            l4_3, l4_4,
        ]
    }
}

struct U;
impl U {
    // pub const iid: TUID = const { unsafe { core::mem::zeroed() } };
    pub const UID: UID = UID;
}

mod private {
    use std::{marker::PhantomData, os::raw::c_void};

    use crate::plugininterfaces::base::funknown::TUID;

    /// Marker trait for types that don't have IID
    trait Void: Sized {
        type Type;
    }
    impl<T> Void for T {
        type Type = c_void;
    }
    type VoidType<T> = <T as Void>::Type;

    /// Marker trait for types that have an IID member
    pub trait HasIIDType {}

    /// Main trait to detect if a type has an IID member
    pub trait HasIIDTypeMarker {
        const HAS_IID_TYPE: bool;
        type HasIID: Bool;
        // type HasIID: Bool<Self::HAS_IID_TYPE>;
    }

    trait Bool {}
    trait TrueType: Bool {}
    trait FalseType: Bool {}
    pub struct True;
    impl Bool for True {}
    impl TrueType for True {}

    // impl Bool for True {}
    pub struct False;
    impl Bool for False {}
    impl FalseType for False {}

    impl<T> HasIIDTypeMarker for T {
        default const HAS_IID_TYPE: bool = false;
        default type HasIID = False;
    }

    impl<T> HasIIDTypeMarker for T
    where
        T: HasIIDType,
    {
        const HAS_IID_TYPE: bool = true;
        type HasIID = True;
    }

    struct GetTUID<T>(PhantomData<T>);

    pub trait HasIIDMemberVariable {}
    impl<T> HasIIDMemberVariable for T
    where
        T: HasIIDTypeMarker,
        <T as HasIIDTypeMarker>::HasIID: TrueType,
    {
    }

    // pub trait Uses
    pub trait HasIIDMemberFunction {}
    impl<T> HasIIDMemberFunction for T
    where
        T: HasIIDTypeMarker,
        <T as HasIIDTypeMarker>::HasIID: FalseType,
    {
    }

    impl<T> GetTUID<T>
    where
        T: HasIIDTypeMarker,
        <T as HasIIDTypeMarker>::HasIID: TrueType,
    {
        fn get_tuid() -> &'static TUID {
            todo!()
        }
    }
}
// fn get_tuid() -> &'static TUID {
//     todo!()
// }
// pub trait HasIID {
//     fn get_tuid() -> &'static TUID;
// }
