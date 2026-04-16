# Minimal io module for embedded Python

import aios_kernel

class FileIO:
    """File I/O wrapper"""
    
    def __init__(self, path, mode='r'):
        self.path = path
        self.mode = mode
        self.fd = None
        self.open()
    
    def open(self):
        """Open the file"""
        flags = 0
        if 'r' in self.mode:
            flags |= 0x01
        if 'w' in self.mode:
            flags |= 0x02
        
        self.fd = aios_kernel.syscall(10, self.path, flags)
    
    def read(self, size=-1):
        """Read from file"""
        if size < 0:
            size = 4096  # Read in chunks
        
        data = bytearray()
        while True:
            chunk = aios_kernel.syscall(12, self.fd, size)
            if not chunk:
                break
            data.extend(chunk)
            if size > 0:
                break
        
        return bytes(data)
    
    def write(self, data):
        """Write to file"""
        if isinstance(data, str):
            data = data.encode('utf-8')
        
        return aios_kernel.syscall(13, self.fd, data, len(data))
    
    def close(self):
        """Close file"""
        if self.fd is not None:
            aios_kernel.syscall(11, self.fd)
            self.fd = None
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()

def open(path, mode='r'):
    """Open a file"""
    return FileIO(path, mode)
