//! Python bindings for AI-OS kernel APIs
//! Exposes kernel functionality to Python code (File I/O + Audio)

use super::ffi::*;
use crate::{println, fs};
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::{string::String, vec::Vec};

lazy_static! {
    static ref SECURITY_GROUPS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn has_group(name: &str) -> bool {
    let groups = SECURITY_GROUPS.lock();
    groups.iter().any(|g| g == name)
}

fn permissions_to_mode(p: fs::Permissions) -> u16 {
    let mut mode: u16 = 0;
    if p.user_read { mode |= 0o400; }
    if p.user_write { mode |= 0o200; }
    if p.user_execute { mode |= 0o100; }
    if p.group_read { mode |= 0o040; }
    if p.group_write { mode |= 0o020; }
    if p.group_execute { mode |= 0o010; }
    if p.other_read { mode |= 0o004; }
    if p.other_write { mode |= 0o002; }
    if p.other_execute { mode |= 0o001; }
    mode
}

// Helpers to extract Python args
unsafe fn py_arg_tuple_get_str(args: PyObject, index: isize) -> Option<String> {
    let obj = PyTuple_GetItem(args, index);
    if obj.is_null() { return None; }
    let c = PyUnicode_AsUTF8(obj);
    if c.is_null() { return None; }
    let mut len = 0;
    while *c.add(len) != 0 { len += 1; }
    let slice = core::slice::from_raw_parts(c, len);
    core::str::from_utf8(slice).ok().map(|s| s.to_string())
}

unsafe fn py_arg_tuple_get_long(args: PyObject, index: isize) -> Option<i64> {
    let obj = PyTuple_GetItem(args, index);
    if obj.is_null() { return None; }
    Some(PyLong_AsLong(obj))
}

// ---------------- File I/O bindings ----------------

#[no_mangle]
pub extern "C" fn aios_open(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let path = match py_arg_tuple_get_str(args, 0) { Some(s) => s, None => return PyLong_FromLong(-1) };
        let flags = py_arg_tuple_get_long(args, 1).unwrap_or(0) as u32;
        match fs::vfs::open(&path, flags) {
            Ok(fd) => PyLong_FromLong(fd as i64),
            Err(_) => PyLong_FromLong(-1),
        }
    }
}

#[no_mangle]
pub extern "C" fn aios_close(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let fd = py_arg_tuple_get_long(args, 0).unwrap_or(-1) as u32;
        match fs::vfs::close(fd) {
            Ok(_) => PyLong_FromLong(0),
            Err(_) => PyLong_FromLong(-1),
        }
    }
}

#[no_mangle]
pub extern "C" fn aios_read(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let fd = py_arg_tuple_get_long(args, 0).unwrap_or(-1) as u32;
        let size = py_arg_tuple_get_long(args, 1).unwrap_or(0) as usize;
        let mut buf = vec![0u8; size];
        match fs::vfs::read(fd, &mut buf) {
            Ok(n) => PyBytes_FromStringAndSize(buf.as_ptr(), n as isize),
            Err(_) => PyBytes_FromStringAndSize(core::ptr::null(), 0),
        }
    }
}

#[no_mangle]
pub extern "C" fn aios_write(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let fd = py_arg_tuple_get_long(args, 0).unwrap_or(-1) as u32;
        let data_obj = PyTuple_GetItem(args, 1);
        if data_obj.is_null() { return PyLong_FromLong(-1); }
        let data_ptr = PyBytes_AsString(data_obj);
        if data_ptr.is_null() { return PyLong_FromLong(-1); }
        let data_len = PyBytes_Size(data_obj) as usize;
        let slice = core::slice::from_raw_parts(data_ptr as *const u8, data_len);
        match fs::vfs::write(fd, slice) {
            Ok(n) => PyLong_FromLong(n as i64),
            Err(_) => PyLong_FromLong(-1),
        }
    }
}

#[no_mangle]
pub extern "C" fn aios_stat(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let path = match py_arg_tuple_get_str(args, 0) { Some(s) => s, None => return PyDict_New() };
        match fs::vfs::stat(&path) {
            Ok(st) => {
                let d = PyDict_New();
                PyDict_SetItem(d, PyUnicode_FromString(b"size\0".as_ptr()), PyLong_FromLong(st.size as i64));
                let ty = match st.file_type {
                    fs::FileType::Regular => b"regular\0",
                    fs::FileType::Directory => b"directory\0",
                    fs::FileType::Symlink => b"symlink\0",
                    fs::FileType::Device => b"device\0",
                };
                PyDict_SetItem(d, PyUnicode_FromString(b"file_type\0".as_ptr()), PyUnicode_FromString(ty.as_ptr()));
                let mode = permissions_to_mode(st.permissions);
                let mut buf = [0u8; 4];
                let s = format!("{:03o}", mode);
                let bytes = s.as_bytes();
                buf[..bytes.len()].copy_from_slice(bytes);
                PyDict_SetItem(d, PyUnicode_FromString(b"mode\0".as_ptr()), PyLong_FromLong(mode as i64));
                PyDict_SetItem(d, PyUnicode_FromString(b"permissions\0".as_ptr()), PyUnicode_FromString(buf.as_ptr()));
                PyDict_SetItem(d, PyUnicode_FromString(b"created\0".as_ptr()), PyLong_FromLong(st.created as i64));
                PyDict_SetItem(d, PyUnicode_FromString(b"modified\0".as_ptr()), PyLong_FromLong(st.modified as i64));
                PyDict_SetItem(d, PyUnicode_FromString(b"accessed\0".as_ptr()), PyLong_FromLong(st.accessed as i64));
                d
            }
            Err(_) => PyDict_New(),
        }
    }
}

#[no_mangle]
pub extern "C" fn aios_readdir(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let path = match py_arg_tuple_get_str(args, 0) { Some(s) => s, None => return PyList_New(0) };
        match fs::vfs::readdir(&path) {
            Ok(entries) => {
                let list = PyList_New(entries.len() as isize);
                for (i, e) in entries.iter().enumerate() {
                    let mut v = e.name.as_bytes().to_vec();
                    v.push(0);
                    let pyname = PyUnicode_FromString(v.as_ptr());
                    PyList_SetItem(list, i as isize, pyname);
                }
                list
            }
            Err(_) => PyList_New(0),
        }
    }
}

#[no_mangle]
pub extern "C" fn aios_create(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let path = match py_arg_tuple_get_str(args, 0) { Some(s) => s, None => return PyLong_FromLong(-1) };
        let is_dir = py_arg_tuple_get_long(args, 1).unwrap_or(0) != 0;
        let ty = if is_dir { fs::FileType::Directory } else { fs::FileType::Regular };
        match fs::vfs::create(&path, ty) {
            Ok(_) => PyLong_FromLong(0),
            Err(_) => PyLong_FromLong(-1),
        }
    }
}

#[no_mangle]
pub extern "C" fn aios_delete(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let path = match py_arg_tuple_get_str(args, 0) { Some(s) => s, None => return PyLong_FromLong(-1) };
        match fs::vfs::delete(&path) {
            Ok(_) => PyLong_FromLong(0),
            Err(_) => PyLong_FromLong(-1),
        }
    }
}

// ---------------- Audio bindings (basic) ----------------

#[no_mangle]
pub extern "C" fn aios_set_groups(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        let groups_csv = match py_arg_tuple_get_str(args, 0) {
            Some(s) => s,
            None => return PyLong_FromLong(-1),
        };
        let mut groups = SECURITY_GROUPS.lock();
        groups.clear();
        for raw in groups_csv.split(',') {
            let g = raw.trim();
            if !g.is_empty() {
                groups.push(g.to_string());
            }
        }
        PyLong_FromLong(0)
    }
}

#[no_mangle]
pub extern "C" fn aios_audio_open_mic(_self: PyObject, _args: PyObject) -> PyObject {
    unsafe {
        if !has_group("voice") {
            return PyLong_FromLong(-1);
        }
        let r = crate::drivers::audio::open_microphone();
        match r { Ok(id) => PyLong_FromLong(id as i64), Err(_) => PyLong_FromLong(-1) }
    }
}

#[no_mangle]
pub extern "C" fn aios_audio_capture(_self: PyObject, args: PyObject) -> PyObject {
    // Capture duration in ms, at 16kHz mono 16-bit (32 KB/s)
    unsafe {
        if !has_group("voice") {
            return PyBytes_FromStringAndSize(core::ptr::null(), 0);
        }
        let duration_ms = py_arg_tuple_get_long(args, 0).unwrap_or(1000) as usize;
        let bytes = duration_ms * 32; // 32 bytes per ms at 16kHz*2B
        let mut buf = vec![0u8; bytes];
        match crate::drivers::audio::capture_audio(&mut buf) {
            Ok(n) => PyBytes_FromStringAndSize(buf.as_ptr(), n as isize),
            Err(_) => PyBytes_FromStringAndSize(core::ptr::null(), 0),
        }
    }
}

#[no_mangle]
pub extern "C" fn aios_audio_play(_self: PyObject, args: PyObject) -> PyObject {
    unsafe {
        if !has_group("voice") {
            return PyLong_FromLong(-1);
        }
        let data_obj = PyTuple_GetItem(args, 0);
        if data_obj.is_null() { return PyLong_FromLong(-1); }
        let data_ptr = PyBytes_AsString(data_obj);
        if data_ptr.is_null() { return PyLong_FromLong(-1); }
        let len = PyBytes_Size(data_obj) as usize;
        let slice = core::slice::from_raw_parts(data_ptr as *const u8, len);
        match crate::drivers::audio::play_audio(slice) {
            Ok(n) => PyLong_FromLong(n as i64),
            Err(_) => PyLong_FromLong(-1),
        }
    }
}

// ---------------- Module registration ----------------

static mut METHODS: [PyMethodDef; 13] = [
    PyMethodDef { ml_name: b"open\0".as_ptr(),    ml_meth: Some(aios_open),          ml_flags: METH_VARARGS, ml_doc: b"open(path, flags) -> fd\0".as_ptr() },
    PyMethodDef { ml_name: b"close\0".as_ptr(),   ml_meth: Some(aios_close),         ml_flags: METH_VARARGS, ml_doc: b"close(fd) -> 0/-1\0".as_ptr() },
    PyMethodDef { ml_name: b"read\0".as_ptr(),    ml_meth: Some(aios_read),          ml_flags: METH_VARARGS, ml_doc: b"read(fd, size) -> bytes\0".as_ptr() },
    PyMethodDef { ml_name: b"write\0".as_ptr(),   ml_meth: Some(aios_write),         ml_flags: METH_VARARGS, ml_doc: b"write(fd, data) -> n\0".as_ptr() },
    PyMethodDef { ml_name: b"stat\0".as_ptr(),    ml_meth: Some(aios_stat),          ml_flags: METH_VARARGS, ml_doc: b"stat(path) -> dict\0".as_ptr() },
    PyMethodDef { ml_name: b"readdir\0".as_ptr(), ml_meth: Some(aios_readdir),       ml_flags: METH_VARARGS, ml_doc: b"readdir(path) -> list\0".as_ptr() },
    PyMethodDef { ml_name: b"create\0".as_ptr(),  ml_meth: Some(aios_create),        ml_flags: METH_VARARGS, ml_doc: b"create(path, is_dir) -> 0/-1\0".as_ptr() },
    PyMethodDef { ml_name: b"delete\0".as_ptr(),  ml_meth: Some(aios_delete),        ml_flags: METH_VARARGS, ml_doc: b"delete(path) -> 0/-1\0".as_ptr() },
    PyMethodDef { ml_name: b"set_groups\0".as_ptr(),     ml_meth: Some(aios_set_groups),     ml_flags: METH_VARARGS, ml_doc: b"set_groups(csv) -> 0/-1\0".as_ptr() },
    PyMethodDef { ml_name: b"audio_open_mic\0".as_ptr(), ml_meth: Some(aios_audio_open_mic), ml_flags: METH_VARARGS, ml_doc: b"open microphone -> id/-1\0".as_ptr() },
    PyMethodDef { ml_name: b"audio_capture\0".as_ptr(),  ml_meth: Some(aios_audio_capture),  ml_flags: METH_VARARGS, ml_doc: b"audio_capture(duration_ms) -> bytes\0".as_ptr() },
    PyMethodDef { ml_name: b"audio_play\0".as_ptr(),     ml_meth: Some(aios_audio_play),     ml_flags: METH_VARARGS, ml_doc: b"audio_play(bytes) -> n/-1\0".as_ptr() },
    PyMethodDef { ml_name: core::ptr::null(), ml_meth: None, ml_flags: 0, ml_doc: core::ptr::null() },
];

#[no_mangle]
pub extern "C" fn PyInit_aios_kernel() -> PyObject {
    unsafe {
        static mut MODULE_DEF: PyModuleDef = PyModuleDef {
            m_base: PyModuleDef_Base { ob_refcnt: 1, ob_type: core::ptr::null_mut(), m_init: core::ptr::null_mut(), m_index: 0, m_copy: core::ptr::null_mut() },
            m_name: b"aios_kernel\0".as_ptr(),
            m_doc: b"AI-OS kernel bindings\0".as_ptr(),
            m_size: -1,
            m_methods: METHODS.as_ptr(),
            m_slots: core::ptr::null_mut(),
            m_traverse: core::ptr::null_mut(),
            m_clear: core::ptr::null_mut(),
            m_free: core::ptr::null_mut(),
        };
        PyModule_Create2(&MODULE_DEF as *const PyModuleDef, PYTHON_API_VERSION)
    }
}

/// Initialize Python bindings
pub fn init() {
    println!("[PYTHON] Registering aios_kernel (File I/O + Audio)...");
    unsafe {
        let module = PyInit_aios_kernel();
        if !module.is_null() {
            let sys_mod = PyImport_ImportModule(b"sys\0".as_ptr());
            if !sys_mod.is_null() {
                let dict = PyModule_GetDict(sys_mod);
                let mods = PyDict_GetItem(dict, PyUnicode_FromString(b"modules\0".as_ptr()));
                if !mods.is_null() {
                    PyDict_SetItem(mods, PyUnicode_FromString(b"aios_kernel\0".as_ptr()), module);
                }
            }
        }
    }
    println!("[PYTHON] ✓ aios_kernel registered");
}
