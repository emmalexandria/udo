use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;
use std::{ffi::c_void, mem::MaybeUninit};

use anyhow::Result;
use nix::libc;
use nix::unistd::User;
use pam_sys::{PamConversation, PamHandle, PamMessage, PamResponse, wrapped::start};
use pam_sys::{PamFlag, PamItemType, PamReturnCode, wrapped::*};

const PAM_SUCCESS: c_int = 0;
const PAM_BUF_ERR: c_int = 5;
const PAM_CONV_ERR: c_int = 19;
const PROMPT_ECHO_OFF: c_int = 1;
const PROMPT_ECHO_ON: c_int = 2;
const ERR_MSG: c_int = 3;
const TEXT_INFO: c_int = 4;

pub enum PamResult {}

extern "C" fn pam_conversation(
    num_msg: c_int,
    msg: *mut *mut PamMessage,
    resp: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> i32 {
    unsafe {
        // Allocate response array
        let responses =
            libc::calloc(num_msg as usize, std::mem::size_of::<PamResponse>()) as *mut PamResponse;

        if responses.is_null() {
            return PAM_BUF_ERR;
        }

        // Get the password from appdata_ptr
        let password = appdata_ptr as *const c_char;

        for i in 0..num_msg {
            let message = *msg.offset(i as isize);
            let msg_style = (*message).msg_style;

            match msg_style {
                PROMPT_ECHO_OFF | PROMPT_ECHO_ON => {
                    // Copy the password
                    let pass_len = libc::strlen(password);
                    let resp_str = libc::malloc(pass_len + 1) as *mut c_char;

                    if resp_str.is_null() {
                        // Clean up on failure
                        for j in 0..i {
                            let resp_ptr = responses.offset(j as isize);
                            if !(*resp_ptr).resp.is_null() {
                                libc::free((*resp_ptr).resp as *mut c_void);
                            }
                        }
                        libc::free(responses as *mut c_void);
                        return PAM_BUF_ERR;
                    }

                    libc::strcpy(resp_str, password);
                    (*responses.offset(i as isize)).resp = resp_str;
                    (*responses.offset(i as isize)).resp_retcode = 0;
                }
                ERR_MSG | TEXT_INFO => {
                    // For informational messages, we don't need to respond
                    (*responses.offset(i as isize)).resp = ptr::null_mut();
                    (*responses.offset(i as isize)).resp_retcode = 0;
                }
                _ => {
                    // Unknown message style
                    for j in 0..i {
                        let resp_ptr = responses.offset(j as isize);
                        if !(*resp_ptr).resp.is_null() {
                            libc::free((*resp_ptr).resp as *mut c_void);
                        }
                    }
                    libc::free(responses as *mut c_void);
                    return PAM_CONV_ERR;
                }
            }
        }

        *resp = responses;
        PAM_SUCCESS
    }
}

/// Authenticate a user with PAM
pub fn authenticate_user(username: &str, password: &str, service: &str) -> Result<bool, String> {
    unsafe {
        let mut pamh: *mut PamHandle = ptr::null_mut();

        // Convert strings to C strings
        let c_username = CString::new(username).map_err(|e| format!("Invalid username: {}", e))?;
        let c_password = CString::new(password).map_err(|e| format!("Invalid password: {}", e))?;
        let c_service = CString::new(service).map_err(|e| format!("Invalid service: {}", e))?;

        // Setup PAM conversation structure
        let conv = PamConversation {
            conv: Some(pam_conversation),
            data_ptr: c_password.as_ptr() as *mut c_void,
        };

        // Start PAM session
        let mut ret = start(
            c_service.to_str().unwrap(),
            Some(c_username.to_str().unwrap()),
            &conv,
            &mut pamh,
        );

        if ret != PamReturnCode::SUCCESS {
            return Err(format!(
                "pam_start failed: {}",
                get_pam_error(&mut *pamh, ret)
            ));
        }

        let rhost = CString::new("localhost").unwrap();
        let rhost_raw = rhost.as_ptr() as *const c_void;
        set_item(&mut *pamh, PamItemType::RHOST, &*rhost_raw);

        // Authenticate the user
        ret = authenticate(&mut *pamh, PamFlag::NONE);
        if ret != PamReturnCode::SUCCESS {
            end(&mut *pamh, ret);
            return Err(format!(
                "Authentication failed: {}, {}",
                get_pam_error(&mut *pamh, ret),
                ret
            ));
        }

        // Validate account (check if account is valid, not expired, etc.)
        ret = acct_mgmt(&mut *pamh, PamFlag::NONE);
        if ret != PamReturnCode::SUCCESS {
            end(&mut *pamh, ret);
            return Err(format!(
                "Account validation failed: {}",
                get_pam_error(&mut *pamh, ret)
            ));
        }

        // Clean up
        end(&mut *pamh, PamReturnCode::SUCCESS);

        Ok(true)
    }
}

/// Get human-readable PAM error message
fn get_pam_error(pamh: &mut PamHandle, error_code: PamReturnCode) -> String {
    let error_cstr = strerror(pamh, error_code);
    if error_cstr.is_none() {
        return format!("Unknown error (code: {})", error_code);
    }
    error_cstr.unwrap().to_string()
}
