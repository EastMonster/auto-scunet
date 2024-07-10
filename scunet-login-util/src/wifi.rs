//! 检测设备的 WiFi 是否已连接到 SCUNET

use crate::LoginError;

pub fn check_wifi(on_boot: bool) -> Result<(), LoginError> {
    #[cfg(target_os = "windows")]
    return windows::check_wifi(on_boot);

    #[cfg(not(target_os = "windows"))]
    return _others::check_wifi(on_boot);
}

#[cfg(target_os = "windows")]
mod windows {
    use std::{ffi::c_void, thread::sleep, time::Duration};

    use windows::{
        core::PCSTR,
        Win32::{
            Foundation::HANDLE,
            NetworkManagement::WiFi::{
                wlan_intf_opcode_current_connection, WlanCloseHandle, WlanEnumInterfaces,
                WlanFreeMemory, WlanOpenHandle, WlanQueryInterface, WLAN_CONNECTION_ATTRIBUTES,
                WLAN_INTERFACE_INFO_LIST,
            },
        },
    };

    use crate::LoginError;

    // ref: https://www.reddit.com/r/rust/comments/zhv63t/comment/izpp30r
    pub fn check_wifi(on_boot: bool) -> Result<(), LoginError> {
        unsafe {
            let max_attempt = if on_boot { 5 } else { 1 };
            let mut attempts = 0;
            let mut last_error;
            loop {
                let mut negotiated_version: u32 = 0;
                let mut wlan_handle: HANDLE = HANDLE::default();

                let res = WlanOpenHandle(2, None, &mut negotiated_version, &mut wlan_handle);
                if res != 0 {
                    last_error = LoginError::WiFiStatusError("无法打开 WLAN 句柄", res);
                    attempts += 1;
                    if attempts >= max_attempt {
                        return Err(last_error);
                    }
                    sleep(Duration::from_secs(1));
                    continue;
                }

                let mut info_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();

                let res = WlanEnumInterfaces(wlan_handle, None, &mut info_list_ptr);
                if res != 0 {
                    last_error = LoginError::WiFiStatusError("无法获取 WLAN 接口列表", res);
                    attempts += 1;
                    if attempts >= max_attempt {
                        return Err(last_error);
                    }
                    sleep(Duration::from_secs(1));
                    continue;
                }

                let guid = (*info_list_ptr).InterfaceInfo[0].InterfaceGuid;

                let mut data_size: u32 = 0;
                let mut ppdata: *mut c_void = std::ptr::null_mut();

                let res = WlanQueryInterface(
                    wlan_handle,
                    &guid,
                    wlan_intf_opcode_current_connection,
                    None,
                    &mut data_size,
                    &mut ppdata,
                    None,
                );
                if res != 0 {
                    last_error = LoginError::WiFiStatusError("无法获取 WLAN 连接属性", res);
                    attempts += 1;
                    if attempts >= max_attempt {
                        return Err(last_error);
                    }
                    sleep(Duration::from_secs(1));
                    continue;
                }

                let wlan_connection_attributes = ppdata as *mut WLAN_CONNECTION_ATTRIBUTES;

                let ssid_arr = (*wlan_connection_attributes)
                    .wlanAssociationAttributes
                    .dot11Ssid
                    .ucSSID;

                let ssid = PCSTR::from_raw(ssid_arr.as_ptr()).to_string().unwrap();

                WlanCloseHandle(wlan_handle, None);
                WlanFreeMemory(info_list_ptr as _);
                WlanFreeMemory(ppdata);

                if ssid != "SCUNET" {
                    last_error = LoginError::NotConnectedToScunet;
                    attempts += 1;
                    if attempts >= max_attempt {
                        return Err(last_error);
                    }
                    sleep(Duration::from_secs(1));
                    continue;
                } else {
                    return Ok(());
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod _others {
    use crate::LoginError;

    pub fn check_wifi(_on_boot: bool) -> Result<(), LoginError> {
        Ok(())
    }
}
