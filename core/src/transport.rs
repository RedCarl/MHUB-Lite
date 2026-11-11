use crate::error::DeviceError;
use hidapi::{HidApi, HidDevice};

/// 一层很薄的「能收发 64 字节包」的抽象。
///
/// 真实设备走 hidapi；演示模式走一个按同一套帧格式应答的内存设备。
/// 这样上面所有读取/校验/重试逻辑在两种模式下走的是同一条代码路径。
pub trait Transport: Send {
    fn write_packet(&mut self, payload: &[u8]) -> Result<(), DeviceError>;
    fn read_packet(&mut self, timeout_ms: i32) -> Result<[u8; 64], DeviceError>;
    fn flush_input(&mut self);
}

pub struct HidTransport {
    device: HidDevice,
}

impl HidTransport {
    pub fn open(vid: u16, pid: u16, serial: Option<&str>) -> Result<Self, DeviceError> {
        let api = HidApi::new()?;
        let target = api.device_list().find(|d| {
            d.vendor_id() == vid
                && d.product_id() == pid
                && d.usage_page() == crate::catalog::CONFIG_USAGE_PAGE
                && d.usage() == crate::catalog::CONFIG_USAGE
                && match serial {
                    Some(want) => d.serial_number() == Some(want),
                    None => true,
                }
        });

        let info = target.ok_or_else(|| {
            DeviceError::not_found("设备不在线：请检查 USB 连接或 2.4G 接收器是否插好")
        })?;
        let device = info.open_device(&api)?;
        Ok(Self { device })
    }
}

impl Transport for HidTransport {
    fn write_packet(&mut self, payload: &[u8]) -> Result<(), DeviceError> {
        self.flush_input();
        let mut pkt = [0u8; 65];
        pkt[0] = 0x00; // Report ID
        let len = payload.len().min(64);
        pkt[1..1 + len].copy_from_slice(&payload[..len]);
        self.device.write(&pkt)?;
        Ok(())
    }

    fn read_packet(&mut self, timeout_ms: i32) -> Result<[u8; 64], DeviceError> {
        let mut buf = [0u8; 65];
        let n = self.device.read_timeout(&mut buf, timeout_ms)?;
        if n == 0 {
            return Err(DeviceError::timeout("设备没有在超时内回应"));
        }
        let mut out = [0u8; 64];
        if buf[0] == 0xAA {
            out.copy_from_slice(&buf[..64]);
        } else if buf[1] == 0xAA {
            out.copy_from_slice(&buf[1..65]);
        } else {
            out.copy_from_slice(&buf[..64]);
        }
        Ok(out)
    }

    fn flush_input(&mut self) {
        let mut buf = [0u8; 65];
        while let Ok(n) = self.device.read_timeout(&mut buf, 0) {
            if n == 0 {
                break;
            }
        }
    }
}
