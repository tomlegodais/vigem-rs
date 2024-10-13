use crate::constants::GUID_DEVINTERFACE_BUSENUM_VIGEM;
use crate::errors::VigemError;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
use winapi::um::fileapi::{CreateFileA, OPEN_EXISTING};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::setupapi::{SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsA, SetupDiGetDeviceInterfaceDetailA, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, SP_DEVICE_INTERFACE_DATA, SP_DEVICE_INTERFACE_DETAIL_DATA_A};
use winapi::um::winbase::{FILE_FLAG_NO_BUFFERING, FILE_FLAG_OVERLAPPED, FILE_FLAG_WRITE_THROUGH};
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, HANDLE};

#[derive(Debug)]
pub struct VigemClient {
    driver_handle: HANDLE,
}

impl VigemClient {
    pub fn new() -> Self {
        Self {
            driver_handle: INVALID_HANDLE_VALUE,
        }
    }

    pub fn connect(&mut self) -> anyhow::Result<(), VigemError> {
        unsafe {
            let device_info_set = SetupDiGetClassDevsA(
                &GUID_DEVINTERFACE_BUSENUM_VIGEM,
                ptr::null(),
                ptr::null_mut(),
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            );

            if device_info_set == INVALID_HANDLE_VALUE {
                return Err(VigemError::DeviceNotFound);
            }

            let mut device_interface_data = std::mem::zeroed::<SP_DEVICE_INTERFACE_DATA>();
            device_interface_data.cbSize = size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;

            let mut member_index = 0;
            let mut required_size = 0;

            while SetupDiEnumDeviceInterfaces(
                device_info_set,
                ptr::null_mut(),
                &GUID_DEVINTERFACE_BUSENUM_VIGEM,
                member_index,
                &mut device_interface_data) != 0
            {
                SetupDiGetDeviceInterfaceDetailA(
                    device_info_set,
                    &mut device_interface_data,
                    ptr::null_mut(),
                    0,
                    &mut required_size,
                    ptr::null_mut(),
                );

                let layout = Layout::from_size_align(required_size as usize, align_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>()).unwrap();
                let detail_data = alloc(layout) as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_A;
                if detail_data.is_null() {
                    return Err(VigemError::AllocationError);
                }

                (*detail_data).cbSize = size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32;

                if SetupDiGetDeviceInterfaceDetailA(
                    device_info_set,
                    &mut device_interface_data,
                    detail_data,
                    required_size,
                    ptr::null_mut(),
                    ptr::null_mut()) != 0
                {
                    self.driver_handle = CreateFileA(
                        (*detail_data).DevicePath.as_ptr(),
                        GENERIC_READ | GENERIC_WRITE,
                        FILE_SHARE_READ | FILE_SHARE_WRITE,
                        ptr::null_mut(),
                        OPEN_EXISTING,
                        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH | FILE_FLAG_OVERLAPPED,
                        ptr::null_mut(),
                    );

                    if self.driver_handle != INVALID_HANDLE_VALUE {
                        dealloc(detail_data as *mut u8, layout);
                        break;
                    }
                }

                member_index += 1;
                dealloc(detail_data as *mut u8, layout);
            }

            if self.driver_handle == INVALID_HANDLE_VALUE {
                return Err(VigemError::DeviceNotFound);
            }

            Ok(())
        }
    }

    pub fn disconnect(&self) {
        unsafe {
            if self.driver_handle != INVALID_HANDLE_VALUE {
                CloseHandle(self.driver_handle);
            }
        }
    }
}

impl Drop for VigemClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}