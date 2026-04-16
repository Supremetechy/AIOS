//! Python C API FFI bindings
//! Low-level bindings to CPython C API

#![allow(dead_code)]

/// Opaque Python object pointer
pub type PyObject = *mut u8;

/// Python thread state
pub type PyThreadState = *mut u8;

/// Python thread state
pub type PyThreadState = *mut u8;

pub const METH_VARARGS: i32 = 0x0001;
pub type PyCFunction = extern "C" fn(self_obj: PyObject, args: PyObject) -> PyObject;

#[repr(C)]
pub struct PyMethodDef {
    pub ml_name: *const u8,
    pub ml_meth: Option<PyCFunction>,
    pub ml_flags: i32,
    pub ml_doc: *const u8,
}

#[repr(C)]
pub struct PyModuleDef_Base {
    pub ob_refcnt: isize,
    pub ob_type: *mut u8,
    pub m_init: *mut u8,
    pub m_index: isize,
    pub m_copy: *mut u8,
}

#[repr(C)]
pub struct PyModuleDef {
    pub m_base: PyModuleDef_Base,
    pub m_name: *const u8,
    pub m_doc: *const u8,
    pub m_size: isize,
    pub m_methods: *const PyMethodDef,
    pub m_slots: *mut u8,
    pub m_traverse: *mut u8,
    pub m_clear: *mut u8,
    pub m_free: *mut u8,
}

pub const PYTHON_API_VERSION: i32 = 1013; // CPython 3.12


// Python C API function declarations (will be linked with libpython)

#[link(name = "python3.12", kind = "static")]
extern "C" {
    // Initialization and finalization
    pub fn Py_Initialize();
    pub fn Py_Finalize();
    pub fn Py_IsInitialized() -> i32;
    pub fn Py_SetProgramName(name: *const u8);
    pub fn Py_GetVersion() -> *const u8;
    
    // Module management
    pub fn PyImport_AddModule(name: *const u8) -> PyObject;
    pub fn PyImport_ImportModule(name: *const u8) -> PyObject;
    pub fn PyModule_GetDict(module: PyObject) -> PyObject;
    pub fn PyModule_Create2(module: *const PyModuleDef, version: i32) -> PyObject;
    pub fn PyModule_AddObject(module: PyObject, name: *const u8, value: PyObject) -> i32;

    // Bytes helpers
    pub fn PyBytes_FromStringAndSize(s: *const u8, len: isize) -> PyObject;
    pub fn PyBytes_AsString(bytes: PyObject) -> *mut u8;
    pub fn PyBytes_Size(bytes: PyObject) -> isize;

    // Tuple helpers
    pub fn PyTuple_Size(tuple: PyObject) -> isize;
    pub fn PyTuple_GetItem(tuple: PyObject, pos: isize) -> PyObject;

    // List helpers
    pub fn PyList_SetItem(list: PyObject, index: isize, item: PyObject) -> i32;
    
    // Code execution
    pub fn PyRun_SimpleString(code: *const u8) -> i32;
    pub fn PyRun_SimpleFile(fp: *mut u8, filename: *const u8) -> i32;
    
    // Object operations
    pub fn PyObject_Str(obj: PyObject) -> PyObject;
    pub fn PyObject_Call(callable: PyObject, args: PyObject, kwargs: PyObject) -> PyObject;
    
    // String operations
    pub fn PyUnicode_FromString(str: *const u8) -> PyObject;
    pub fn PyUnicode_AsUTF8(unicode: PyObject) -> *const u8;
    
    // Dict operations
    pub fn PyDict_New() -> PyObject;
    pub fn PyDict_GetItem(dict: PyObject, key: PyObject) -> PyObject;
    pub fn PyDict_SetItem(dict: PyObject, key: PyObject, value: PyObject) -> i32;
    
    // List operations
    pub fn PyList_New(size: isize) -> PyObject;
    pub fn PyList_Append(list: PyObject, item: PyObject) -> i32;
    
    // Tuple operations
    pub fn PyTuple_New(size: isize) -> PyObject;
    pub fn PyTuple_SetItem(tuple: PyObject, pos: isize, item: PyObject) -> i32;
    
    // Long (integer) operations
    pub fn PyLong_FromLong(value: i64) -> PyObject;
    pub fn PyLong_AsLong(obj: PyObject) -> i64;
    
    // Float operations
    pub fn PyFloat_FromDouble(value: f64) -> PyObject;
    pub fn PyFloat_AsDouble(obj: PyObject) -> f64;
    
    // Error handling
    pub fn PyErr_Occurred() -> PyObject;
    pub fn PyErr_Print();
    pub fn PyErr_Clear();
    
    // Reference counting
    pub fn Py_IncRef(obj: PyObject);
    pub fn Py_DecRef(obj: PyObject);
}

// Safe wrapper functions

pub unsafe fn py_initialize() {
    Py_Initialize();
}

pub unsafe fn py_finalize() {
    Py_Finalize();
}

pub unsafe fn py_is_initialized() -> bool {
    Py_IsInitialized() != 0
}

pub unsafe fn py_set_program_name(name: *const u8) {
    Py_SetProgramName(name);
}

pub unsafe fn py_get_version() -> *const u8 {
    Py_GetVersion()
}

pub unsafe fn py_import_add_module(name: *const u8) -> PyObject {
    PyImport_AddModule(name)
}

pub unsafe fn py_import_module(name: *const u8) -> PyObject {
    PyImport_ImportModule(name)
}

pub unsafe fn py_module_get_dict(module: PyObject) -> PyObject {
    PyModule_GetDict(module)
}

pub unsafe fn py_run_simple_string(code: *const u8) -> i32 {
    PyRun_SimpleString(code)
}

pub unsafe fn py_object_str(obj: PyObject) -> PyObject {
    PyObject_Str(obj)
}

pub unsafe fn py_unicode_from_string(str: *const u8) -> PyObject {
    PyUnicode_FromString(str)
}

pub unsafe fn py_unicode_as_utf8(unicode: PyObject) -> *const u8 {
    PyUnicode_AsUTF8(unicode)
}

pub unsafe fn py_dict_new() -> PyObject {
    PyDict_New()
}

pub unsafe fn py_dict_get_item(dict: PyObject, key: PyObject) -> PyObject {
    PyDict_GetItem(dict, key)
}

pub unsafe fn py_dict_set_item(dict: PyObject, key: PyObject, value: PyObject) -> i32 {
    PyDict_SetItem(dict, key, value)
}

pub unsafe fn py_long_from_long(value: i64) -> PyObject {
    PyLong_FromLong(value)
}

pub unsafe fn py_long_as_long(obj: PyObject) -> i64 {
    PyLong_AsLong(obj)
}

pub unsafe fn py_err_occurred() -> bool {
    !PyErr_Occurred().is_null()
}

pub unsafe fn py_err_print() {
    PyErr_Print();
}

pub unsafe fn py_err_clear() {
    PyErr_Clear();
}
