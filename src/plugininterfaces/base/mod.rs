use bytemuck::{Pod, Zeroable};
use std::{
    fmt,
    marker::PhantomData,
    mem::{transmute, transmute_copy, MaybeUninit},
    ops::Deref,
    os::raw::c_void,
    ptr::NonNull,
};

#[derive(Debug)]
pub enum EInterface {
    BadCast,
    BadQuery,
    Pointer,

    InitFailed,
}
impl fmt::Display for EInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EInterface::BadCast => f.write_str("Bad interface cast"),
            EInterface::BadQuery => f.write_str("Bad interface query"),
            EInterface::Pointer => f.write_str("Tried to write interface to null-pointer"),
            EInterface::InitFailed => f.write_str("IPluginBase initialization failed"),
        }
    }
}
impl std::error::Error for EInterface {}

// pub trait RuntimeName {
//     const NAME: &'static str = "";
// }

/// Interface trait 'inspired' by windows-rs
/// # Safety
/// It's safe, trust me
pub unsafe trait Interface: Sized + Clone {
    type Vtable;
    #[allow(non_upper_case_globals)]
    const iid: FUID;

    #[allow(non_upper_case_globals)]
    const unknown: bool = true;

    /// A reference to the interface's vtable
    ///
    #[inline(always)]
    fn vtable(&self) -> &Self::Vtable {
        // SAFETY: Implementor of the trait must guarantee that `Self` is castable to its vtable
        unsafe { self.assume_vtable::<Self>() }
    }

    /// Cast this interface as a reference to the supplied interfaces `Vtable`
    ///
    /// # Safety
    ///
    /// This is safe if `T` is an equivalent `Interface` to `Self` or a super `Interface`.
    /// In other words, `T::vtable` mut be equivalent to the beginning of `Self::Vtable`.    
    #[inline(always)]
    unsafe fn assume_vtable<T: Interface>(&self) -> &T::Vtable {
        &**(self.as_raw() as *mut *mut T::Vtable)
    }

    /// Returns the raw `Interface` pointer.
    /// The resulting pointer continues to be owned by the `Interface` implementation.
    #[inline(always)]
    fn as_raw(&self) -> *mut c_void {
        // SAFETY: Implementors of this trait must guarantee that the implementing type has a pointer in-memory representation
        unsafe { transmute_copy(self) }
    }
    /// Creates an `Interface` by taking ownership of the raw `Interface` pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be owned by the caller and represent a valid `Interface` pointer.
    /// In other words, it must point to a `Vtable` beginning with the `FUnknown` function pointers
    /// and match the `Vtable` of `Interface`
    unsafe fn from_raw(raw: *mut c_void) -> Self {
        transmute_copy(&raw)
    }
    /// Attempts to cast the current `Interface` to another `Interface` using `Self::query`.
    #[inline(always)]
    fn cast<T: Interface>(&self) -> Result<T, EInterface> {
        unsafe {
            // This way we can propagate error and not drop
            let mut result = MaybeUninit::<Option<T>>::zeroed();

            self.query(&T::iid, result.as_mut_ptr() as _)?;

            if let Some(obj) = result.assume_init() {
                Ok(obj)
            } else {
                Err(EInterface::Pointer)
            }
        }
    }
    /// Query an `Interface` on this `Interface`
    ///
    /// # Safety
    ///
    /// `interface` must be a non-null, valid pointer for writing an interface pointer.
    #[inline(always)]
    unsafe fn query(
        &self,
        iid: *const FUID,
        interface: *mut *mut c_void,
    ) -> Result<(), EInterface> {
        if Self::unknown {
            (self.assume_vtable::<funknown::FUnknown>().query_interface)(
                self.as_raw(),
                iid,
                interface,
            )
        } else {
            Err(EInterface::BadQuery)
        }
    }
    /// Creates an `InterfaceRef` for this reference.
    /// The `InterfaceRef` tracks lifetimes statically
    /// and eliminates the need for dynamic reference count adjustments (AddRef/Release)
    fn as_ref(&self) -> InterfaceRef<'_, Self> {
        InterfaceRef::from_interface(self)
    }
}

pub struct InterfaceRef<'a, I>(NonNull<c_void>, PhantomData<&'a I>);
impl<'a, I: Interface> InterfaceRef<'a, I> {
    /// Creates an `InterfaceRef` from an `Interface` reference.
    /// Safely associates the lifetime of the `Interface` reference with the
    /// `'i` parameter of `InterfaceRef`.
    /// This allows for lifetime checking _without_ calling AddRef/Release on the underlying
    /// lifetime.
    #[inline(always)]
    pub fn from_interface(interface: &I) -> Self {
        unsafe {
            // SAFETY: new_unchecked() should be valid because `Interface::as_raw` should always
            // return a non-null pointer.
            Self(NonNull::new_unchecked(interface.as_raw()), PhantomData)
        }
    }
    /// Calls AddRef on the underlying `Interface` and returns an "owned" (counted) reference.
    pub fn to_owned(self) -> I {
        (*self).clone()
    }
}
impl<'a, 'i: 'a, I: Interface> From<&'i I> for InterfaceRef<'a, I> {
    #[inline(always)]
    fn from(interface: &'i I) -> Self {
        InterfaceRef::from_interface(interface)
    }
}
impl<'a, I: Interface> Deref for InterfaceRef<'a, I> {
    type Target = I;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { transmute(self) }
    }
}

pub mod coreiids;
pub mod funknown;
pub mod ipluginbase;

#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

type TUIDType = u8;
const TUID_LENGTH: usize = 16_usize;
#[allow(clippy::upper_case_acronyms)]
pub type TUID = [TUIDType; TUID_LENGTH];

mod helpers {
    pub(super) const fn make_long(b1: u8, b2: u8, b3: u8, b4: u8) -> u32 {
        (b1 as u32) << 24 | (b2 as u32) << 16 | (b3 as u32) << 8 | (b4 as u32)
    }
}

/// Handling 16 byte globally unique identifiers.
///
/// Each interface declares its identifier as static
/// member inside the interface namespace, e.g. [`FUnknown::iid`]
///
#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FUID {
    pub data: TUID,
}
unsafe impl Zeroable for FUID {}
unsafe impl Pod for FUID {}
impl From<TUID> for FUID {
    #[inline(always)]
    fn from(value: TUID) -> Self {
        let mut data: TUID = [0_u8; TUID_LENGTH];
        unsafe { std::ptr::copy_nonoverlapping(&value as *const _, &mut data as *mut _, 1) };
        Self { data }
    }
}
impl From<[u32; 4]> for FUID {
    #[inline(always)]
    fn from(value: [u32; 4]) -> Self {
        let data: TUID = [
            (value[0] & 0x00_00_00_FF) as u8,
            ((value[0] & 0x00_00_FF_00) >> 8) as u8,
            ((value[0] & 0x00_FF_00_00) >> 16) as u8,
            ((value[0] & 0xFF_00_00_00) >> 24) as u8,
            ((value[1] & 0x00_FF_00_00) >> 16) as u8,
            ((value[1] & 0xFF_00_00_00) >> 24) as u8,
            (value[1] & 0x00_00_00_FF) as u8,
            ((value[1] & 0x00_00_FF_00) >> 8) as u8,
            ((value[2] & 0xFF_00_00_00) >> 24) as u8,
            ((value[2] & 0x00_FF_00_00) >> 16) as u8,
            ((value[2] & 0x00_00_FF_00) >> 8) as u8,
            (value[2] & 0x00_00_00_FF) as u8,
            ((value[3] & 0xFF_00_00_00) >> 24) as u8,
            ((value[3] & 0x00_FF_00_00) >> 16) as u8,
            ((value[3] & 0x00_00_FF_00) >> 8) as u8,
            (value[3] & 0x00_00_00_FF) as u8,
        ];
        Self { data }
    }
}
// impl<const N: usize> From<[u8; N]> for Fuid {
//     fn from(value: [u8; N]) -> Self {
//         assert_eq!(N, size_of::<Tuid>(), "Only TUID compatible arrays allowed");
//         let mut data = Tuid::zeroed();
//         unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), &mut data as *mut _ as *mut _, N) };
//         Self { data }
//     }
// }
impl PartialEq for FUID {
    fn eq(&self, other: &Self) -> bool {
        funknown::FUnknownPrivate::iid_equal(
            self.data.as_ptr() as *const _,
            other.data.as_ptr() as *const _,
        )
    }
    // fn ne(&self, other: &Self) -> bool {
    //     !FUnknownPrivate::iid_equal(
    //         self.data.as_ptr() as *const _,
    //         other.data.as_ptr() as *const _,
    //     )
    // }
}
impl PartialOrd for FUID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.data.as_slice().cmp(other.data.as_slice()))
    }
}
impl AsRef<TUID> for FUID {
    fn as_ref(&self) -> &TUID {
        &self.data
    }
}
impl fmt::Display for FUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let g = self.data.as_ptr() as *const GUID;

        let r = unsafe { g.as_ref().unwrap() };

        f.write_fmt(format_args!(
            "{:#08x}{:#04x}{:#04x}{:#02x}{:#02x}{:#02x}{:#02x}{:#02x}{:#02x}{:#02x}{:#02x}",
            r.data1,
            r.data2,
            r.data3,
            r.data4[0],
            r.data4[1],
            r.data4[2],
            r.data4[3],
            r.data4[4],
            r.data4[5],
            r.data4[6],
            r.data4[7],
        ))
    }
}
impl FUID {
    pub fn new(l1: u32, l2: u32, l3: u32, l4: u32) -> Self {
        todo!()
    }
    pub fn generate() -> Option<Self> {
        todo!()
    }
    pub fn is_valid(&self) -> bool {
        todo!()
    }
    pub fn get_long_1(&self) -> u32 {
        helpers::make_long(self.data[3], self.data[2], self.data[1], self.data[0])
    }
    pub fn get_long_2(&self) -> u32 {
        helpers::make_long(self.data[5], self.data[4], self.data[7], self.data[6])
    }
    pub fn get_long_3(&self) -> u32 {
        helpers::make_long(self.data[8], self.data[9], self.data[10], self.data[11])
    }
    pub fn get_long_4(&self) -> u32 {
        helpers::make_long(self.data[12], self.data[13], self.data[14], self.data[15])
    }
}
pub mod macros {
    #[macro_export]
    macro_rules! inline_uid {
        ($l1:literal, $l2:literal, $l3:literal, $l4:literal) => {
            [
                ($l1 as u32 & 0x00_00_00_FF) as u8,
                (($l1 as u32 & 0x00_00_FF_00) >> 8) as u8,
                (($l1 as u32 & 0x00_FF_00_00) >> 16) as u8,
                (($l1 as u32 & 0xFF_00_00_00) >> 24) as u8,
                (($l2 as u32 & 0x00_FF_00_00) >> 16) as u8,
                (($l2 as u32 & 0xFF_00_00_00) >> 24) as u8,
                ($l2 as u32 & 0x00_00_00_FF) as u8,
                (($l2 as u32 & 0x00_00_FF_00) >> 8) as u8,
                (($l3 as u32 & 0xFF_00_00_00) >> 24) as u8,
                (($l3 as u32 & 0x00_FF_00_00) >> 16) as u8,
                (($l3 as u32 & 0x00_00_FF_00) >> 8) as u8,
                ($l3 as u32 & 0x00_00_00_FF) as u8,
                (($l4 as u32 & 0xFF_00_00_00) >> 24) as u8,
                (($l4 as u32 & 0x00_FF_00_00) >> 16) as u8,
                (($l4 as u32 & 0x00_00_FF_00) >> 8) as u8,
                ($l4 as u32 & 0x00_00_00_FF) as u8,
            ]
        };
        ($l1:expr, $l2:expr, $l3:expr, $l4:expr) => {
            [
                ($l1 as u32 & 0x00_00_00_FF) as u8,
                (($l1 as u32 & 0x00_00_FF_00) >> 8) as u8,
                (($l1 as u32 & 0x00_FF_00_00) >> 16) as u8,
                (($l1 as u32 & 0xFF_00_00_00) >> 24) as u8,
                (($l2 as u32 & 0x00_FF_00_00) >> 16) as u8,
                (($l2 as u32 & 0xFF_00_00_00) >> 24) as u8,
                ($l2 as u32 & 0x00_00_00_FF) as u8,
                (($l2 as u32 & 0x00_00_FF_00) >> 8) as u8,
                (($l3 as u32 & 0xFF_00_00_00) >> 24) as u8,
                (($l3 as u32 & 0x00_FF_00_00) >> 16) as u8,
                (($l3 as u32 & 0x00_00_FF_00) >> 8) as u8,
                ($l3 as u32 & 0x00_00_00_FF) as u8,
                (($l4 as u32 & 0xFF_00_00_00) >> 24) as u8,
                (($l4 as u32 & 0x00_FF_00_00) >> 16) as u8,
                (($l4 as u32 & 0x00_00_FF_00) >> 8) as u8,
                ($l4 as u32 & 0x00_00_00_FF) as u8,
            ]
        };
    }
    pub use inline_uid;
    #[macro_export(local_inner_macros)]
    macro_rules! declare_class_iid {
        ($class_name:tt, $l1:literal, $l2:literal, $l3:literal, $l4:literal) => {
            paste::paste! {
                #[allow(non_upper_case_globals, dead_code)]
                pub const [<$class_name _iid>]: $crate::plugininterfaces::base::TUID = inline_uid!($l1, $l2, $l3, $l4);
            }
        };
    }
    pub use declare_class_iid;
    #[macro_export]
    macro_rules! def_class_iid {
        ($class_name:tt) => {
            impl $class_name {
                #[allow(non_upper_case_globals, dead_code)]
                const iid: $crate::plugininterfaces::base::FUID =
                    $crate::plugininterfaces::base::FUID {
                        data: paste::paste! {[<$class_name _iid >] },
                    };
            }
            // impl HasIID for $class_name {
            //     fn get_tuid() -> &'static TUID {
            //         $class_name::iid.0
            //     }
            // }

            // unsafe impl $crate::plugininterfaces::base::funknown::Interface for $class_name {
            //     type Vtable = paste::paste! { [<$class_name Vtable>] };
            //     const iid: $crate::plugininterfaces::base::FUID =
            //         $crate::plugininterfaces::base::FUID {
            //             data: paste::paste! { [<$class_name _iid>] },
            //         };
            // }
        };
    }
    pub use def_class_iid;

    macro_rules! def_vtable_field {
        ($field_name:tt: $field_type:ty) => {
            pub $field_name: $field_type
        };
    }

    #[macro_export]
    macro_rules! declare_class_deref {
        ($class_name:ident, $base:ty) => {
            impl std::ops::Deref for $class_name {
                type Target = $base;
                fn deref(&self) -> &Self::Target {
                    unsafe { core::mem::transmute(self) }
                }
            }
        };
    }
    pub use declare_class_deref;

    #[macro_export(local_inner_macros)]
    macro_rules! declare_class_vtable_deref {
        ($class_name: ident, $base:ty) => {
            paste::paste! {
                declare_class_deref!(
                    [<$class_name Vtable>],
                    [<$base Vtable>]
                );
            }
        };
    }
    pub use declare_class_vtable_deref;
    #[macro_export(local_inner_macros)]
    macro_rules! declare_class_vtable {
        // (FUnknown, $($field_name:tt: $field_type:ty),+) => {
        //     paste::paste! {
        //         #[repr(C)]
        //         pub struct [ <$class_name Vtable> ] {
        //             $(pub $field_name: $field_type,)*
        //         }
        //     }
        // };
        ($class_name:ident , base FUnknown, $(function $function_name:tt: ($($arg_name:tt: $arg_type:ty),*)$(-> $return_type:ty)?),+) => {
            paste::paste! {
                #[repr(C)]
                pub struct [ <$class_name Vtable> ] {
                    base: FUnknownVtable,
                    $(pub $function_name: unsafe fn(this: *mut c_void, $($arg_name: $arg_type),*)$(-> $return_type)?,)*
                }
                pub trait [<$class_name Impl>]: Sized {
                    // type Impl;

                    // fn get_impl(&self) -> &Self::Impl;

                    $(unsafe fn $function_name(&self $(, $arg_name: $arg_type)*)$(-> $return_type)?;)*

                    const INNER_OFFSET_IN_POINTERS: usize;
                }
                declare_class_deref!($class_name, FUnknown);
                // declare_class_vtable_deref!($class_name, FUnknown);
            }
        };
        ($class_name:ident $(, base $base:ty)? $(, bound $bound:ty)?, $(function $function_name:tt: ($($arg_name:tt: $arg_type:ty),*)$(-> $return_type:ty)?),+) => {
            paste::paste! {
                #[repr(C)]
                pub struct [ <$class_name Vtable> ] {
                    $(base: [ <$base Vtable> ],)*
                    $(pub $function_name: unsafe fn(this: *mut c_void, $($arg_name: $arg_type),*)$(-> $return_type)?,)*
                }
                // impl [<$class_name Vtable>] {
                //     pub const fn new<Identity: $crate::plugininterfaces::base::pluginreexports::FUnknownImpl, const OFFSET: isize>() -> Self {
                //         Self { base: }
                //     }
                // }
                // pub trait [<$class_name Impl>]: Sized $(+ [<$base Impl>])* {
                //     type Impl;

                //     fn get_impl(&self) -> &<Self as [<$class_name Impl>]>::Impl;

                //     $(unsafe fn $function_name(&self $(, $arg_name: $arg_type)*)$(-> $return_type)?;)*

                //     const INNER_OFFSET_IN_POINTERS: usize;
                // }
                pub trait [<$class_name Impl>]: Sized $(+ [<$bound Impl>])* {
                    $(unsafe fn $function_name(&self $(, $arg_name: $arg_type)*)$(-> $return_type)?;)*
                }
                $( declare_class_deref!($class_name, $bound); )*
            }
        };
    }
    pub use declare_class_vtable;

    #[macro_export(local_inner_macros)]
    macro_rules! declare_interface {
        ($class_name:tt) => {
            #[repr(transparent)]
            #[derive(Clone, PartialEq, Eq, Debug)]
            pub struct $class_name($crate::plugininterfaces::base::funknown::FUnknown);
            unsafe impl $crate::plugininterfaces::base::Interface for $class_name {
                type Vtable = paste::paste! { [<$class_name Vtable> ] };
                const iid: $crate::plugininterfaces::base::FUID =
                    $crate::plugininterfaces::base::FUID {
                        data: paste::paste! { [<$class_name _iid>] },
                    };
            }
        };
    }
    pub use declare_interface;

    #[macro_export(local_inner_macros)]
    macro_rules! interface_hierarchy {
        ($child:ident, $parent:ty) => {
            impl From<&$child> for &$parent {
                fn from(value: &$child) -> Self {
                    unsafe { std::mem::transmute(value) }
                }
            }
            impl From<$child> for $parent {
                fn from(value: $child) -> Self {
                    unsafe { std::mem::transmute(value) }
                }
            }
            // impl std::ops::Deref for $child {
            //     type Target = $parent;
            //     fn deref(&self) -> &Self::Target {
            //         unsafe { std::mem::transmute(self) }
            //     }
            // }
        };
        ($child:ident, $first:ty, $($rest:ty),+) => {
            interface_hierarchy!($child, $first);
            interface_hierarchy!($child, $($rest),+);
        }
    }
    pub use interface_hierarchy;
}

pub mod pluginreexports {
    macro_rules! reexport {
        ($crate_name:tt, $class_name:tt) => {
            //use super::$crate_name;

            paste::paste! {
                pub use $crate_name::{$class_name, [<$class_name Vtable>], [<$class_name _iid>]};
            }
            //use super::$crate_name;
        };
        ($crate_name:tt, $first:tt, $($rest:tt),+) => {
            //use super::$crate_name;
            reexport!($crate_name, $first);
            reexport!($crate_name, $($rest),+);
        };
    }
    pub use super::{funknown, ipluginbase, FUID};
    pub use funknown::FUnknownImpl;
    reexport!(funknown, FUnknown);
    reexport!(ipluginbase, IPluginBase, IPluginFactory, IPluginFactory2);
}
