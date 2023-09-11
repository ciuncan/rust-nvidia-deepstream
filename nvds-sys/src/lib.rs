pub mod ffi;

// let mut ptr: *mut ffi::NvDsFrameMeta = ...; // initialize the pointer
// let slice: &mut [ffi::NvDsFrameMeta] = unsafe { std::slice::from_raw_parts_mut(ptr, 1) };
// let reference: &mut ffi::NvDsFrameMeta = &mut slice[0];


macro_rules! impl_glist_iterator {
    ($list_name:ident, $item_type:ty) => {
        pub struct $list_name {
            list: *mut ffi::GList,
        }

        impl From<*mut ffi::GList> for $list_name {
            fn from(list: *mut ffi::GList) -> Self {
                Self { list }
            }
        }

        impl Iterator for $list_name {
            type Item = *mut $item_type;

            fn next(&mut self) -> Option<Self::Item> {
                let list_ref = self.list;
                if list_ref.is_null() {
                    None
                } else {
                    let current_list = unsafe { *list_ref };
                    let data = current_list.data as *mut $item_type;
                    self.list = current_list.next;
                    Some(data)
                }
            }
        }
    };
}

impl_glist_iterator!(NvDsClassifierMetaList, ffi::NvDsClassifierMeta);
impl_glist_iterator!(NvDisplayMetaList, ffi::NvDsDisplayMeta);
impl_glist_iterator!(NvDsFrameMetaList, ffi::NvDsFrameMeta);
impl_glist_iterator!(NvDsLabelInfoList, ffi::NvDsLabelInfo);
impl_glist_iterator!(NvDsMetaList, ffi::NvDsMeta);
impl_glist_iterator!(NvDsObjectMetaList, ffi::NvDsObjectMeta);
impl_glist_iterator!(NvDsUserMetaList, ffi::NvDsUserMeta);

#[test]
fn test_glist_next() {
    impl_glist_iterator!(TestIntIter, i32);

    // Create a list with two elements
    let mut data1 = 2;
    let mut data2 = 1;
    let mut underlying_list_node_1 = ffi::GList {
        data: &data1 as *const i32 as ffi::gpointer,
        next: std::ptr::null_mut(),
        prev: std::ptr::null_mut(),
    };
    let mut underlying_list_node_2 = ffi::GList {
        data: &data2 as *const i32 as ffi::gpointer,
        next: std::ptr::null_mut(),
        prev: &mut underlying_list_node_1 as *mut _,
    };
    underlying_list_node_1.next = &mut underlying_list_node_2 as *mut _;

    // Test the next method
    let mut iter = TestIntIter::from(&mut underlying_list_node_1 as *mut _);
    assert_eq!(iter.next(), Some(&mut data1 as *mut i32));
    assert_eq!(iter.next(), Some(&mut data2 as *mut i32));
    assert_eq!(iter.next(), None);
}
