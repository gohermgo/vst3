use super::{macros::*, pluginreexports::*, EInterface, Interface, TUID};
use std::{ffi::CStr, mem::transmute_copy, ops, os::raw::c_void, ptr::null_mut};

declare_interface!(IPluginBase);
interface_hierarchy!(IPluginBase, FUnknown);
declare_class_iid!(
    IPluginBase,
    0x2288_8DD8,
    0x156E_45AE,
    0x8358_B348,
    0x0819_0625
);
impl IPluginBase {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn initialize(&self, context: *mut c_void) -> Result<(), EInterface> {
        unsafe { (self.vtable().initialize)(transmute_copy(self), context) }
    }
    pub fn terminate(&self) {
        unsafe { (self.vtable().terminate)(transmute_copy(self)) }
    }
}
declare_class_vtable!(
    IPluginBase,
    base FUnknown,
    function initialize: (/*this, */context: *mut c_void) -> Result<(), EInterface>,
    function terminate: (/*this*/)
);

impl IPluginBaseVtable {
    pub const fn new<Identity: FUnknownImpl, const OFFSET: isize>() -> Self
    where
        Identity: IPluginBaseImpl,
    {
        unsafe fn initialize<Identity: FUnknownImpl, const OFFSET: isize>(
            this: *mut c_void,
            context: *mut c_void,
        ) -> Result<(), EInterface>
        where
            Identity: IPluginBaseImpl,
        {
            let this: &Identity = &*((this as *mut *mut c_void).offset(OFFSET) as *const Identity);
            IPluginBaseImpl::initialize(this, context)
        }
        unsafe fn terminate<Identity: FUnknownImpl, const OFFSET: isize>(this: *mut c_void)
        where
            Identity: IPluginBaseImpl,
        {
            let this: &Identity = &*((this as *mut *mut c_void).offset(OFFSET) as *const Identity);
            IPluginBaseImpl::terminate(this)
        }
        Self {
            base: FUnknownVtable::new::<Identity, OFFSET>(),
            initialize: initialize::<Identity, OFFSET>,
            terminate: terminate::<Identity, OFFSET>,
        }
    }
}
#[repr(u32)]
pub enum FactoryFlags {
    None,

    ClassesDiscardable = 1 << 0,

    LicenseCheck = 1 << 1,

    /// Component will not be unloaded until process exit
    ComponentNonDiscardable = 1 << 3,

    Unicode = 1 << 4,
}
impl ops::BitOr<FactoryFlags> for u32 {
    type Output = u32;
    fn bitor(self, rhs: FactoryFlags) -> Self::Output {
        self | rhs as u32
    }
}
impl ops::BitOrAssign<FactoryFlags> for u32 {
    fn bitor_assign(&mut self, rhs: FactoryFlags) {
        *self = *self | rhs;
    }
}
#[repr(C)]
pub struct FactoryInfo {
    vendor: [u8; 64],
    url: [u8; 256],
    email: [u8; 128],
    flags: u32,
}
#[repr(u32)]
pub enum ClassCardinality {
    ManyInstances = 0x7FFF_FFFF,
}
impl ops::BitOr<ClassCardinality> for u32 {
    type Output = u32;
    fn bitor(self, rhs: ClassCardinality) -> Self::Output {
        self | rhs as u32
    }
}
#[repr(C)]
pub struct PClassInfo {
    /// Class ID 16 Byte class GUID
    cid: TUID,
    cardinality: i32,
    category: [u8; PClassInfo::kCategorySize],
    name: [u8; PClassInfo::kNameSize],
}
impl PClassInfo {
    const kManyInstances: i32 = 0x7FFF_FFFF;
    const kCategorySize: usize = 32;
    const kNameSize: usize = 64;
    pub const fn new(
        cid: TUID,
        cardinality: i32,
        category_str: &'static CStr,
        name_str: &'static CStr,
    ) -> Self {
        let mut category = [0_u8; PClassInfo::kCategorySize];
        let category_bytes = category_str.to_bytes();
        let mut category_idx = 0;
        while category_idx < category_bytes.len() {
            category[category_idx] = category_bytes[category_idx];
            category_idx += 1;
        }

        let mut name = [0_u8; PClassInfo::kNameSize];
        let name_bytes = name_str.to_bytes();
        let mut name_idx = 0;
        while name_idx < name_bytes.len() {
            name[name_idx] = name_bytes[name_idx];
            name_idx += 1;
        }

        PClassInfo {
            cid,
            cardinality,
            category,
            name,
        }
    }
}
declare_interface!(IPluginFactory);
interface_hierarchy!(IPluginFactory, FUnknown);
declare_class_iid!(
    IPluginFactory,
    0x7A4D_811C,
    0x5211_4A1F,
    0xAED9_D2EE,
    0x0B43_BF9F
);
impl IPluginFactory {
    pub fn get_factory_info(&self) -> FactoryInfo {
        todo!()
    }
    pub fn count_classes(&self) -> u32 {
        todo!()
    }
    pub fn get_class_info(&self, index: u32) -> Option<PClassInfo> {
        todo!()
    }
    pub fn create_instance<I: Interface>(&self, cid: TUID, iid: &FUID) -> Result<I, EInterface> {
        todo!()
    }
}
declare_class_vtable!(
    IPluginFactory,
    base FUnknown,
    function get_factory_info: (/*this*/) -> Result<FactoryInfo, EInterface>,
    function count_classes: (/*this*/) -> u32,
    function get_class_info: (/*this,*/ index: u32) -> Result<PClassInfo,EInterface>
);
impl IPluginFactoryVtable {
    pub const fn new<Identity: FUnknownImpl, const OFFSET: isize>() -> Self
    where
        Identity: IPluginFactoryImpl,
    {
        unsafe fn get_factory_info<Identity: FUnknownImpl, const OFFSET: isize>(
            this: *mut c_void,
        ) -> Result<FactoryInfo, EInterface>
        where
            Identity: IPluginFactoryImpl,
        {
            let this: &Identity = &*((this as *mut *mut c_void).offset(OFFSET) as *const Identity);
            IPluginFactoryImpl::get_factory_info(this)
        }
        unsafe fn count_classes<Identity: FUnknownImpl, const OFFSET: isize>(
            this: *mut c_void,
        ) -> u32
        where
            Identity: IPluginFactoryImpl,
        {
            let this: &Identity = &*((this as *mut *mut c_void).offset(OFFSET) as *const Identity);
            IPluginFactoryImpl::count_classes(this)
        }
        unsafe fn get_class_info<Identity: FUnknownImpl, const OFFSET: isize>(
            this: *mut c_void,
            index: u32,
        ) -> Result<PClassInfo, EInterface>
        where
            Identity: IPluginFactoryImpl,
        {
            let this: &Identity = &*((this as *mut *mut c_void).offset(OFFSET) as *const Identity);
            IPluginFactoryImpl::get_class_info(this, index)
        }
        Self {
            base: FUnknownVtable::new::<Identity, OFFSET>(),
            get_factory_info: get_factory_info::<Identity, OFFSET>,
            count_classes: count_classes::<Identity, OFFSET>,
            get_class_info: get_class_info::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &FUID) -> bool {
        iid == &<IPluginFactory as Interface>::iid
    }
}
#[repr(C)]
pub struct PClassInfo2 {
    base: PClassInfo,
    class_flags: u32,
    sub_categories: [u8; PClassInfo2::kSubCategoriesSize],
    vendor: [u8; PClassInfo2::kVendorSize],
    version: [u8; PClassInfo2::kVersionSize],
    sdk_version: [u8; PClassInfo2::kVersionSize],
}
impl PClassInfo2 {
    const kSubCategoriesSize: usize = 128;
    const kVendorSize: usize = 64;
    const kVersionSize: usize = 64;
    // pub const fn new(cid: TUID, cardinality: i32, )
}
declare_class_iid!(
    IPluginFactory2,
    0x0007_B650,
    0xF24B_4C0B,
    0xA464_EDB9,
    0xF00B_2ABB
);
declare_interface!(IPluginFactory2);
interface_hierarchy!(IPluginFactory2, IPluginFactory, FUnknown);
declare_class_vtable!(IPluginFactory2,
    base IPluginFactory,
    bound IPluginFactory,
    function get_class_info_2: (/* this ,*/ index: u32) -> Result<PClassInfo2, EInterface>
);
impl IPluginFactory2Vtable {
    pub const fn new<Identity: FUnknownImpl, const OFFSET: isize>() -> Self
    where
        Identity: IPluginFactory2Impl,
    {
        unsafe fn get_class_info_2<Identity: FUnknownImpl, const OFFSET: isize>(
            this: *mut c_void,
            index: u32,
        ) -> Result<PClassInfo2, EInterface>
        where
            Identity: IPluginFactory2Impl,
        {
            let this: &Identity = &*((this as *mut *mut c_void).offset(OFFSET) as *const Identity);
            IPluginFactory2Impl::get_class_info_2(this, index)
        }

        Self {
            base: IPluginFactoryVtable::new::<Identity, OFFSET>(),
            get_class_info_2: get_class_info_2::<Identity, OFFSET>,
        }
    }

    pub fn matches(iid: &FUID) -> bool {
        iid == &<IPluginFactory2 as Interface>::iid || iid == &<IPluginFactory as Interface>::iid
    }
}
unsafe fn sm() {
    let x = IPluginFactory2::from_raw(null_mut());
    x.as_ref();
    todo!()
}
