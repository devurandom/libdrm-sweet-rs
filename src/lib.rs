/*
 * This library is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

extern crate libdrm_sys;

use std::ffi::CStr;
use std::ptr;
use libdrm_sys as drm;

pub struct PlatformBusInfo {
    pub fullname: String,
}

pub struct Host1xBusInfo {
    pub fullname: String,
}

pub enum DeviceInfo {
    Pci{ bus: drm::drmPciBusInfo, dev: drm::drmPciDeviceInfo },
    Usb{ bus: drm::drmUsbBusInfo, dev: drm::drmUsbDeviceInfo },
    Platform{ bus: PlatformBusInfo, dev: drm::drmPlatformDeviceInfo },
    Host1x{ bus: Host1xBusInfo, dev: drm::drmHost1xDeviceInfo },
}

impl DeviceInfo {
    pub unsafe fn from_sys(sys_dev: &drm::drmDevice) -> Self {
        return match sys_dev.bustype as u32 {
            drm::DRM_BUS_PCI => DeviceInfo::Pci {
                bus: **sys_dev.businfo.pci.as_ref(),
                dev: **sys_dev.deviceinfo.pci.as_ref(),
            },
            drm::DRM_BUS_USB => DeviceInfo::Usb {
                bus: **sys_dev.businfo.usb.as_ref(),
                dev: **sys_dev.deviceinfo.usb.as_ref(),
            },
            drm::DRM_BUS_PLATFORM => DeviceInfo::Platform{
                bus: PlatformBusInfo {
                    // FIXME: The following is unsafe! We know the size of fullname[], but we discard it...
                    fullname: CStr::from_ptr(
                        &(**sys_dev.businfo.platform.as_ref()).fullname
                            as *const i8)
                        .to_string_lossy()
                        .into_owned(),
                },
                dev: **sys_dev.deviceinfo.platform.as_ref(),
            },
            drm::DRM_BUS_HOST1X => DeviceInfo::Host1x{
                bus: Host1xBusInfo {
                    // FIXME: The following is unsafe! We know the size of fullname[], but we discard it...
                    fullname: CStr::from_ptr(
                        &(**sys_dev.businfo.host1x.as_ref()).fullname
                            as *const i8)
                        .to_string_lossy()
                        .into_owned(),
                },
                dev: **sys_dev.deviceinfo.host1x.as_ref(),
            },
            _ => panic!("Unknown bus type: {}", sys_dev.bustype),
        };
    }
}

pub struct DeviceNodes {
    pub primary: Option<String>,
    pub control: Option<String>,
    pub render: Option<String>,
}

impl DeviceNodes {
    pub unsafe fn from_sys(sys_dev: &drm::drmDevice) -> Self {
        let available_nodes = sys_dev.available_nodes as u32;
        let sys_nodes = std::slice::from_raw_parts(sys_dev.nodes, drm::DRM_NODE_MAX as usize);
        let mut nodes = DeviceNodes { primary: None, control: None, render: None };
        if (available_nodes & (1 << drm::DRM_NODE_PRIMARY)) != 0 {
            nodes.primary = Some(CStr::from_ptr(sys_nodes[drm::DRM_NODE_PRIMARY as usize])
                .to_string_lossy()
                .into_owned());
        }
        if (available_nodes & (1 << drm::DRM_NODE_CONTROL)) != 0 {
            nodes.control = Some(CStr::from_ptr(sys_nodes[drm::DRM_NODE_CONTROL as usize])
                .to_string_lossy()
                .into_owned());
        }
        if (available_nodes & (1 << drm::DRM_NODE_RENDER)) != 0 {
            nodes.render = Some(CStr::from_ptr(sys_nodes[drm::DRM_NODE_RENDER as usize])
                .to_string_lossy()
                .into_owned());
        }
        return nodes;
    }
}

pub struct Device {
    pub nodes: DeviceNodes,
    pub info: DeviceInfo,
}

impl Device {
    pub unsafe fn from_sys(sys_dev_ptr: drm::drmDevicePtr) -> Self {
        let sys_dev = sys_dev_ptr.as_ref()
            .expect("Could not convert from drmDevicePtr to reference");
        return Device {
            nodes: DeviceNodes::from_sys(sys_dev),
            info: DeviceInfo::from_sys(sys_dev),
        };
    }
}

pub fn available() -> bool {
    return unsafe { drm::drmAvailable() != 0 };
}

pub fn get_devices() -> Vec<Device> {
    let num_devices : usize = unsafe { drm::drmGetDevices(ptr::null_mut(), 0) as usize };
    let mut sys_devices : Vec<drm::drmDevicePtr> = Vec::with_capacity(num_devices);
    sys_devices.resize(num_devices, ptr::null_mut());
    unsafe { drm::drmGetDevices(sys_devices.as_mut_ptr(), num_devices as i32) };
    let mut devices : Vec<Device> = Vec::with_capacity(num_devices);
    for d in &sys_devices {
        devices.push(unsafe { Device::from_sys(*d) });
    };
    unsafe { drm::drmFreeDevices(sys_devices.as_mut_ptr(), num_devices as i32) };
    return devices;
}
