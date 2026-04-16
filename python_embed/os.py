# Minimal os module for embedded Python
# Provides file and process operations

import aios_kernel

# File operations
def open(path, mode='r'):
    """Open a file"""
    flags = 0
    if 'r' in mode:
        flags |= 0x01  # O_RDONLY
    if 'w' in mode:
        flags |= 0x02  # O_WRONLY
    if 'a' in mode:
        flags |= 0x04  # O_APPEND
    
    return aios_kernel.open(path, flags)

def read(fd, size):
    """Read from file descriptor"""
    return aios_kernel.read(fd, size)

def write(fd, data):
    """Write to file descriptor"""
    if isinstance(data, str):
        data = data.encode('utf-8')
    return aios_kernel.write(fd, data)

def close(fd):
    """Close file descriptor"""
    return aios_kernel.close(fd)

def mkdir(path):
    """Create directory"""
    return aios_kernel.create(path, 1)  # 1 = directory

def listdir(path):
    """List directory contents"""
    return aios_kernel.readdir(path)

def readdir_pretty(path, show_hidden=False):
    """Pretty-print directory contents with stat output"""
    entries = aios_kernel.readdir(path)
    if not entries:
        print("(empty)")
        return []

    def _join(p, name):
        if p.endswith("/"):
            return p + name
        return p + "/" + name

    rows = []
    for name in entries:
        if not show_hidden and name.startswith("."):
            continue
        stat = aios_kernel.stat(_join(path, name)) or {}
        size = stat.get("size", 0)
        ftype = stat.get("file_type", "unknown")
        perm = stat.get("permissions", "")
        modified = stat.get("modified", 0)
        rows.append((name, ftype, perm, modified, size))

    if not rows:
        print("(empty)")
        return []

    name_width = max(len(r[0]) for r in rows)
    type_width = max(len(r[1]) for r in rows)
    perm_width = max(len(str(r[2])) for r in rows)
    print(f"{'NAME'.ljust(name_width)}  {'TYPE'.ljust(type_width)}  {'PERM'.ljust(perm_width)}  MODIFIED  SIZE")
    print(f"{'-'*name_width}  {'-'*type_width}  {'-'*perm_width}  --------  ----")
    for name, ftype, perm, modified, size in sorted(rows, key=lambda r: r[0].lower()):
        print(f"{name.ljust(name_width)}  {ftype.ljust(type_width)}  {str(perm).ljust(perm_width)}  {modified}  {size}")
    return rows

def remove(path):
    """Remove a file"""
    return aios_kernel.delete(path)

def unlink(path):
    """Remove a file (alias)"""
    return aios_kernel.delete(path)

def rmdir(path):
    """Remove a directory"""
    return aios_kernel.delete(path)

# Process operations
def getpid():
    """Get process ID"""
    return aios_kernel.syscall(4)  # SYS_GETPID

def system(command):
    """Execute system command"""
    # TODO: Implement shell execution
    return 0

# Path operations
def path_exists(path):
    """Check if path exists"""
    try:
        aios_kernel.stat(path)
        return True
    except:
        return False

# Environment
environ = {
    'PATH': '/aios/bin',
    'HOME': '/aios/home',
    'PYTHONHOME': '/aios/lib/python3.12',
}
